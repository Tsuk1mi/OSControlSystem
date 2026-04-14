use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::gesture_os_control::domain::entities::gesture::{GestureResult, TemporalDecisionStatus};
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

#[derive(Clone, Debug)]
pub struct TemporalFilterConfig {
    pub window_size: usize,
    pub stability_threshold: f32,
    pub min_repeat_interval: Duration,
}

impl Default for TemporalFilterConfig {
    fn default() -> Self {
        Self {
            window_size: 7,
            stability_threshold: 0.5,
            min_repeat_interval: Duration::from_millis(520),
        }
    }
}

/// Сглаживание распознавания по кадрам и ограничение частоты срабатываний.
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
            };
        }

        let dominant = majority_gesture(&self.window);
        let Some((gid, ratio)) = dominant else {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Pending,
                stability: 0.0,
                gesture: GestureId::None,
                confidence: sample.confidence,
            };
        };

        if gid == GestureId::None {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Rejected,
                stability: ratio,
                gesture: GestureId::None,
                confidence: sample.confidence,
            };
        }

        if ratio < self.config.stability_threshold {
            return TemporalFilterOutput {
                status: TemporalDecisionStatus::Pending,
                stability: ratio,
                gesture: gid,
                confidence: window_mean_confidence_for(&self.window, gid),
            };
        }

        if let Some((last_id, last_time)) = self.last_confirmed {
            if last_id == gid && now.duration_since(last_time) < self.config.min_repeat_interval {
                return TemporalFilterOutput {
                    status: TemporalDecisionStatus::Pending,
                    stability: ratio,
                    gesture: gid,
                    confidence: window_mean_confidence_for(&self.window, gid),
                };
            }
        }

        self.last_confirmed = Some((gid, now));
        TemporalFilterOutput {
            status: TemporalDecisionStatus::Confirmed,
            stability: ratio,
            gesture: gid,
            confidence: window_mean_confidence_for(&self.window, gid).max(sample.confidence),
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
    if n == 0 {
        0.0
    } else {
        acc / n as f32
    }
}

#[derive(Clone, Debug)]
pub struct TemporalFilterOutput {
    pub status: TemporalDecisionStatus,
    pub stability: f32,
    pub gesture: GestureId,
    pub confidence: f32,
}

fn majority_gesture(window: &VecDeque<GestureResult>) -> Option<(GestureId, f32)> {
    let mut counts: Vec<(GestureId, usize)> = Vec::new();
    for item in window {
        if item.gesture == GestureId::None {
            continue;
        }
        if let Some((_, c)) = counts.iter_mut().find(|(g, _)| *g == item.gesture) {
            *c += 1;
        } else {
            counts.push((item.gesture, 1));
        }
    }
    let total_active = window.iter().filter(|g| g.gesture != GestureId::None).count().max(1);
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(g, c)| (g, c as f32 / total_active as f32))
}
