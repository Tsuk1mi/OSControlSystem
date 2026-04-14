use std::time::Duration;

use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::domain::entities::command::OsCommand;
use crate::gesture_os_control::domain::entities::gesture::{
    AppRunMode, FrameProcessingOutcome, FrameProcessingResult, GestureResult, GestureType,
    TemporalDecisionStatus,
};
use crate::gesture_os_control::domain::entities::landmark::estimate_hand_landmarks;
use crate::gesture_os_control::domain::entities::session_state::FrameProcessingSession;
use crate::gesture_os_control::domain::services::temporal_filter::TemporalFilterOutput;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

pub struct ProcessFrameUseCase;

impl ProcessFrameUseCase {
    /// Полный цикл обработки одного кадра. Исполнение команды ОС — снаружи.
    pub fn execute(
        frame: &FrameDto,
        run_mode: AppRunMode,
        sensitivity: f32,
        gesture_cooldown: Duration,
        session: &mut FrameProcessingSession,
    ) -> FrameProcessingResult {
        session.set_sensitivity(sensitivity);
        session.stats.frames_captured = session.stats.frames_captured.saturating_add(1);

        if let Some(until) = session.gesture_cooldown_until {
            if frame.timestamp < until {
                let raw = GestureResult {
                    gesture: GestureId::None,
                    confidence: 0.0,
                    gesture_type: GestureType::None,
                    timestamp: frame.timestamp,
                };
                session.stats.last_gesture = GestureId::None;
                session.stats.last_confidence = 0.0;
                session.stats.last_filter_status = TemporalDecisionStatus::Rejected;
                session.stats.last_stability = 0.0;
                return FrameProcessingResult {
                    outcome: FrameProcessingOutcome::NoGesture,
                    raw_gesture: raw,
                    filter_stability: 0.0,
                    filter_status: TemporalDecisionStatus::Rejected,
                };
            }
            session.gesture_cooldown_until = None;
        }

        let Some(landmarks) = estimate_hand_landmarks(
            &frame.rgb8,
            frame.width as usize,
            frame.height as usize,
        ) else {
            let raw = GestureResult {
                gesture: GestureId::None,
                confidence: 0.0,
                gesture_type: GestureType::None,
                timestamp: frame.timestamp,
            };
            session.stats.last_gesture = GestureId::None;
            session.stats.last_confidence = 0.0;
            session.stats.last_filter_status = TemporalDecisionStatus::Rejected;
            session.stats.last_stability = 0.0;
            return FrameProcessingResult {
                outcome: FrameProcessingOutcome::NoGesture,
                raw_gesture: raw,
                filter_stability: 0.0,
                filter_status: TemporalDecisionStatus::Rejected,
            };
        };

        let raw = session.classifier.classify(
            &landmarks,
            (frame.width, frame.height),
            frame.timestamp,
        );

        let filter_out = session.temporal.push(raw.clone(), frame.timestamp);
        session.stats.last_filter_status = filter_out.status;
        session.stats.last_stability = filter_out.stability;
        session.stats.last_gesture = filter_out.gesture;
        session.stats.last_confidence = filter_out.confidence;

        let filter_status = filter_out.status;
        let filter_stability = filter_out.stability;

        let outcome = match filter_out.status {
            TemporalDecisionStatus::Pending => FrameProcessingOutcome::GesturePending,
            TemporalDecisionStatus::Rejected => FrameProcessingOutcome::GestureRejected {
                reason: "Нестабильный или пустой жест.".to_owned(),
            },
            TemporalDecisionStatus::Confirmed => Self::handle_confirmed(
                &filter_out,
                run_mode,
                session,
                frame.timestamp,
                gesture_cooldown,
            ),
        };

        FrameProcessingResult {
            outcome,
            raw_gesture: raw,
            filter_stability,
            filter_status,
        }
    }

    fn handle_confirmed(
        filter_out: &TemporalFilterOutput,
        run_mode: AppRunMode,
        session: &mut FrameProcessingSession,
        now: std::time::Instant,
        gesture_cooldown: Duration,
    ) -> FrameProcessingOutcome {
        let command = session.mapper.resolve(run_mode, filter_out.gesture);
        if matches!(command, OsCommand::NoAction) {
            return FrameProcessingOutcome::GestureRejected {
                reason: "Для жеста не найдена команда.".to_owned(),
            };
        }

        let decision = session.safety.evaluate(
            command,
            filter_out.confidence,
            now,
            &mut session.safety_context,
        );

        if !decision.allow {
            return FrameProcessingOutcome::GestureConfirmedCommandDenied {
                reason: decision.reason,
            };
        }

        if !gesture_cooldown.is_zero() {
            session.begin_gesture_cooldown(now, gesture_cooldown);
        }

        FrameProcessingOutcome::CommandReady { command }
    }
}
