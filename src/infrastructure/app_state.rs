//! Сводное состояние подсистемы жестов (расширяется по мере развития UI).

use crate::gesture_os_control::domain::entities::gesture::PipelineGestureStats;

#[derive(Clone, Debug, Default)]
pub struct GestureSubsystemState {
    pub pipeline_stats: Option<PipelineGestureStats>,
}
