use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::domain::entities::gesture::{GestureResult, GestureType};
use crate::gesture_os_control::domain::entities::landmark::estimate_hand_landmarks;
use crate::gesture_os_control::domain::entities::session_state::FrameProcessingSession;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

/// Узкий сценарий: только landmarks + классификатор (без фильтра и команд).
pub struct RecognizeGestureUseCase;

impl RecognizeGestureUseCase {
    pub fn execute(session: &mut FrameProcessingSession, frame: &FrameDto) -> Option<GestureResult> {
        let landmarks = estimate_hand_landmarks(
            &frame.rgb8,
            frame.width as usize,
            frame.height as usize,
        )?;
        Some(session.classifier.classify(
            &landmarks,
            (frame.width, frame.height),
            frame.timestamp,
        ))
    }

    pub fn empty_raw(frame: &FrameDto) -> GestureResult {
        GestureResult {
            gesture: GestureId::None,
            confidence: 0.0,
            gesture_type: GestureType::None,
            timestamp: frame.timestamp,
        }
    }
}
