use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::gesture::TemporalDecisionStatus;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

#[derive(Clone, Debug, Default)]
pub struct GestureDebugFrameDto {
    pub frame_width: u32,
    pub frame_height: u32,
    pub backend_name: String,
    pub backend_label: Option<String>,
    pub landmarks: Vec<[f32; 2]>,
    pub bounding_box: Option<[f32; 4]>,
    pub raw_gesture: GestureId,
    pub raw_confidence: f32,
    pub filter_stability: f32,
    pub filter_status: TemporalDecisionStatus,
    pub filter_reason: String,
    pub detected_motion: Option<[f32; 2]>,
    pub resolved_mode: AppRunMode,
    pub context_summary: String,
}
