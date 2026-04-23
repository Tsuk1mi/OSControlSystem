use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::application::dto::gesture_debug_dto::GestureDebugFrameDto;
use crate::gesture_os_control::domain::entities::gesture::GestureResult;
use crate::gesture_os_control::domain::entities::landmark::HandLandmarks;

#[derive(Clone, Debug)]
pub struct GestureBackendOutput {
    pub landmarks: Option<HandLandmarks>,
    pub direct_gesture: Option<GestureResult>,
    pub debug_frame: GestureDebugFrameDto,
}

pub trait GestureBackendPort: Send {
    fn backend_name(&self) -> &'static str;
    fn process_frame(&mut self, frame: &FrameDto) -> Result<GestureBackendOutput, String>;
}
