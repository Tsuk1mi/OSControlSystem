use std::time::Instant;

use crate::gesture_os_control::domain::entities::gesture::PipelineGestureStats;
use crate::gesture_os_control::domain::services::command_mapper::{CommandMapper, GestureCommandMap};
use crate::gesture_os_control::domain::services::gesture_classifier::{GestureClassifier, GestureClassifierConfig};
use crate::gesture_os_control::domain::services::safety_guard::{SafetyContext, SafetyGuard, SafetyGuardConfig};
use crate::gesture_os_control::domain::services::temporal_filter::{TemporalGestureFilter, TemporalFilterConfig};

/// Сессия обработки: хранит состояние фильтра, классификатора и защиты.
pub struct FrameProcessingSession {
    pub classifier: GestureClassifier,
    pub temporal: TemporalGestureFilter,
    pub mapper: CommandMapper,
    pub safety: SafetyGuard,
    pub safety_context: SafetyContext,
    pub stats: PipelineGestureStats,
    /// После успешной команды до этого момента жесты не распознаём (антиспам).
    pub gesture_cooldown_until: Option<Instant>,
}

impl Default for FrameProcessingSession {
    fn default() -> Self {
        Self::with_command_map(GestureCommandMap::app_defaults())
    }
}

impl FrameProcessingSession {
    pub fn with_command_map(command_map: GestureCommandMap) -> Self {
        Self {
            classifier: GestureClassifier::new(GestureClassifierConfig::default()),
            temporal: TemporalGestureFilter::new(TemporalFilterConfig::default()),
            mapper: CommandMapper::new(command_map),
            safety: SafetyGuard::new(SafetyGuardConfig::default()),
            safety_context: SafetyContext::default(),
            stats: PipelineGestureStats::default(),
            gesture_cooldown_until: None,
        }
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.classifier.set_sensitivity(sensitivity);
    }

    pub fn begin_gesture_cooldown(&mut self, now: Instant, duration: std::time::Duration) {
        self.gesture_cooldown_until = Some(now + duration);
        self.temporal.clear();
        self.classifier.clear_palm_history();
    }
}
