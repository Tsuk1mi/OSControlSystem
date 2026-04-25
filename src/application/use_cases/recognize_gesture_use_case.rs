//! Один кадр: бэкенд → landmarks → `GestureClassifier` (без temporal-фильтра и команд).
use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::application::dto::gesture_debug_dto::GestureDebugFrameDto;
use crate::gesture_os_control::application::ports::input::gesture_backend_port::GestureBackendPort;
use crate::gesture_os_control::domain::entities::gesture::{GestureResult, GestureType};
use crate::gesture_os_control::domain::entities::session_state::FrameProcessingSession;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

#[derive(Clone, Debug)]
pub struct RecognizedFrame {
    pub raw_gesture: GestureResult,
    pub debug_frame: GestureDebugFrameDto,
}

/// Узкий сценарий: только landmarks + классификатор (без фильтра и команд).
pub struct RecognizeGestureUseCase;

impl RecognizeGestureUseCase {
    pub fn execute(
        session: &mut FrameProcessingSession,
        backend: &mut dyn GestureBackendPort,
        frame: &FrameDto,
        sensitivity: f32,
    ) -> Result<RecognizedFrame, String> {
        session.set_sensitivity(sensitivity);
        let output = backend.process_frame(frame)?;
        let raw_gesture = if let Some(direct) = output.direct_gesture {
            direct
        } else if let Some(landmarks) = output.landmarks {
            session
                .classifier
                .classify(&landmarks, (frame.width, frame.height), frame.timestamp)
        } else {
            Self::empty_raw(frame)
        };
        let mut debug_frame = output.debug_frame;
        debug_frame.raw_gesture = raw_gesture.gesture;
        debug_frame.raw_confidence = raw_gesture.confidence;
        debug_frame.detected_motion = session.classifier.last_motion();
        Ok(RecognizedFrame {
            raw_gesture,
            debug_frame,
        })
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
