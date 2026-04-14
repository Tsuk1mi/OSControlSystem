use crate::gesture_os_control::domain::entities::session_state::FrameProcessingSession;

pub struct ManageSessionUseCase;

impl ManageSessionUseCase {
    pub fn reset(session: &mut FrameProcessingSession) {
        *session = FrameProcessingSession::default();
    }

    pub fn set_sensitivity(session: &mut FrameProcessingSession, sensitivity: f32) {
        session.set_sensitivity(sensitivity);
    }
}
