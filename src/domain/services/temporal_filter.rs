use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::gesture_os_control::domain::entities::gesture::{
    GestureResult, GestureType, TemporalDecisionStatus,
};
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

#[derive(Clone, Debug)]
pub struct TemporalFilterConfig {
    pub window_size: usize,
    pub static_stability_threshold: f32,
    pub dynamic_stability_threshold: f32,
    pub static_min_repeat_interval: Duration,
    pub dynamic_min_repeat_interval: Duration,
    pub static_min_hits: usize,
    pub dynamic_min_hits: usize,
}

impl Default for TemporalFilterConfig {
    fn default() -> Self {
        Self {
            window_size: 7,
            static_stability_threshold: 0.56,
            dynamic_stability_threshold: 0.68,
            static_min_repeat_interval: Duration::from_millis(520),
            dynamic_min_repeat_interval: Duration::from_millis(760),
            static_min_hits: 3,
            dynamic_min_hits: 4,
        }
    }
}

// Сглаживание распознавания по кадрам и ограничение частоты срабатываний.
pub struct TemporalGestureFilter {
    config: TemporalFilterConfig,
    window: VecDeque<GestureResult>,
    last_confirmed: Option<(GestureId, Instant)>,
}

impl TemporalGestureFilter {
    pub fn new(config: TemporalFilterConfig) -> Self {
        Self {
            config,
            window: VecDeque::new(),
            last_confirmed: None,
        }
    }

    pub fn clear(&mut self) {
        self.window.clear();
        self.last_confirmed = None;
    }

    pub fn push(&mut self, sample: GestureResult, now: Instant) -> TemporalFilterOutput {
        self.window.push_back(sample.clone());
        while self.window.len() > self.config.window_size {
            self.window.pop_front();
        }

        if self.window.len() < 3 {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Pending,
                stability: 0.0,
                gesture: GestureId::None,
                confidence: 0.0,
                gesture_type: GestureType::None,
                reason: "Собираю окно кадров для стабилизации.".to_owned(),
            };
        }

        let dominant = weighted_majority_gesture(&self.window);
        let Some(dominant) = dominant else {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Rejected,
                stability: 0.0,
                gesture: GestureId::None,
                confidence: sample.confidence,
                gesture_type: GestureType::None,
                reason: "Жест не найден в полном окне кадров.".to_owned(),
            };
        };
        let min_hits = match dominant.gesture_type {
            GestureType::Dynamic => self.config.dynamic_min_hits,
            GestureType::Static => self.config.static_min_hits,
            GestureType::None => usize::MAX,
        };
        if dominant.hits < min_hits {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Pending,
                stability: dominant.ratio,
                gesture: dominant.gesture,
                confidence: dominant.confidence,
                gesture_type: dominant.gesture_type,
                reason: format!(
                    "Недостаточно подтверждений жеста: {}/{}.",
                    dominant.hits, min_hits
                ),
            };
        }
        let required_ratio = match dominant.gesture_type {
            GestureType::Dynamic => self.config.dynamic_stability_threshold,
            GestureType::Static => self.config.static_stability_threshold,
            GestureType::None => 1.0,
        };
        if dominant.ratio < required_ratio {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Pending,
                stability: dominant.ratio,
                gesture: dominant.gesture,
                confidence: dominant.confidence,
                gesture_type: dominant.gesture_type,
                reason: format!(
                    "Стабильность {:.0}% ниже порога {:.0}%.",
                    dominant.ratio * 100.0,
                    required_ratio * 100.0
                ),
            };
        }
        let repeat_interval = match dominant.gesture_type {
            GestureType::Dynamic => self.config.dynamic_min_repeat_interval,
            GestureType::Static => self.config.static_min_repeat_interval,
            GestureType::None => Duration::from_secs(3600),
        };

        if let Some((last_id, last_time)) = self.last_confirmed {
            if last_id == dominant.gesture && now.duration_since(last_time) < repeat_interval {
                return TemporalFilterOutput {
                    status: TemporalDecisionStatus::Pending,
                    stability: dominant.ratio,
                    gesture: dominant.gesture,
                    confidence: dominant.confidence,
                    gesture_type: dominant.gesture_type,
                    reason: "Жест подтверждён слишком недавно.".to_owned(),
                };
            }
        }

        self.last_confirmed = Some((dominant.gesture, now));
        TemporalFilterOutput {
            status: TemporalDecisionStatus::Confirmed,
            stability: dominant.ratio,
            gesture: dominant.gesture,
            confidence: dominant.confidence.max(sample.confidence),
            gesture_type: dominant.gesture_type,
            reason: "Жест стабилен и подтверждён.".to_owned(),
        }
    }
}

fn window_mean_confidence_for(window: &VecDeque<GestureResult>, gesture: GestureId) -> f32 {
    let mut acc = 0.0_f32;
    let mut n = 0_u32;
    for item in window {
        if item.gesture == gesture {
            acc += item.confidence;
            n += 1;
        }
    }
    if n == 0 { 0.0 } else { acc / n as f32 }
}

#[derive(Clone, Debug)]
pub struct TemporalFilterOutput {
    pub status: TemporalDecisionStatus,
    pub stability: f32,
    pub gesture: GestureId,
    pub confidence: f32,
    pub gesture_type: GestureType,
    pub reason: String,
}

#[derive(Clone, Copy, Debug)]
struct DominantGesture {
    gesture: GestureId,
    ratio: f32,
    hits: usize,
    confidence: f32,
    gesture_type: GestureType,
}

fn weighted_majority_gesture(window: &VecDeque<GestureResult>) -> Option<DominantGesture> {
    let mut counts: Vec<(GestureId, GestureType, f32, usize, f32)> = Vec::new();
    let mut total_weight = 0.0_f32;
    for (index, item) in window.iter().enumerate() {
        let weight = (index + 1) as f32;
        total_weight += weight;
        if item.gesture == GestureId::None {
            continue;
        }
        if let Some((_, _, acc_weight, hits, acc_conf)) = counts
            .iter_mut()
            .find(|(gesture, _, _, _, _)| *gesture == item.gesture)
        {
            *acc_weight += weight;
            *hits += 1;
            *acc_conf += item.confidence;
        } else {
            counts.push((item.gesture, item.gesture_type, weight, 1, item.confidence));
        }
    }
    counts
        .into_iter()
        .max_by(|left, right| {
            left.2
                .partial_cmp(&right.2)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(
            |(gesture, gesture_type, match_weight, hits, total_confidence)| DominantGesture {
                gesture,
                ratio: match_weight / total_weight.max(1.0),
                hits,
                confidence: total_confidence / hits.max(1) as f32,
                gesture_type,
            },
        )
}
