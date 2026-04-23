#![allow(dead_code)]

use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::command::OsCommand;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

/// Тип жеста для логики фильтра и отображения.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureType {
    None,
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
pub struct GestureResult {
    pub gesture: GestureId,
    pub confidence: f32,
    pub gesture_type: GestureType,
    pub timestamp: Instant,
}

/// Режим приложения для маппера (контекст).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppRunMode {
    #[default]
    Desktop,
    Media,
    Browser,
}

impl AppRunMode {
    pub fn label_ru(self) -> &'static str {
        match self {
            Self::Desktop => "стол",
            Self::Media => "медиа",
            Self::Browser => "браузер",
        }
    }
}

/// Итог обработки одного кадра (агрегат для UI и оркестратора).
#[derive(Clone, Debug)]
pub enum FrameProcessingOutcome {
    NoGesture,
    GesturePending,
    GestureRejected {
        reason: String,
    },
    GestureConfirmedCommandDenied {
        reason: String,
    },
    /// Команда прошла фильтр и safety-guard; исполнение выполняет внешний слой (например, поток камеры).
    CommandReady {
        command: OsCommand,
    },
    CommandExecuted {
        command: OsCommand,
        execution: super::command::CommandExecutionResult,
    },
    CommandFailed {
        command: OsCommand,
        execution: super::command::CommandExecutionResult,
    },
}

#[derive(Clone, Debug)]
pub struct FrameProcessingResult {
    pub outcome: FrameProcessingOutcome,
    pub raw_gesture: GestureResult,
    pub filter_stability: f32,
    pub filter_status: TemporalDecisionStatus,
    pub filter_reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TemporalDecisionStatus {
    Pending,
    Confirmed,
    #[default]
    Rejected,
}

impl TemporalDecisionStatus {
    pub fn label_ru(self) -> &'static str {
        match self {
            Self::Pending => "ожидание",
            Self::Confirmed => "подтверждён",
            Self::Rejected => "отклонён",
        }
    }
}

/// Статистика для панели «Камера и жесты».
#[derive(Clone, Debug, Default)]
pub struct PipelineGestureStats {
    pub fps_smoothed: f32,
    pub frames_captured: u64,
    pub consecutive_errors: u32,
    pub last_gesture: GestureId,
    pub last_confidence: f32,
    pub last_filter_status: TemporalDecisionStatus,
    pub last_stability: f32,
}
