use std::time::{Duration, Instant};

use crate::gesture_os_control::domain::entities::command::OsCommand;

#[derive(Clone, Debug)]
pub struct SafetyGuardConfig {
    pub min_confidence: f32,
    pub min_command_interval: Duration,
    pub critical_min_confidence: f32,
}

impl Default for SafetyGuardConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.36,
            min_command_interval: Duration::from_millis(280),
            critical_min_confidence: 0.78,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyContext {
    pub last_execution: Option<Instant>,
    pub shutdown_confirmation_pending: bool,
}

#[derive(Clone, Debug)]
pub struct SafetyDecision {
    pub allow: bool,
    pub reason: String,
    #[allow(dead_code)]
    pub needs_extra_confirmation: bool,
}

pub struct SafetyGuard {
    config: SafetyGuardConfig,
}

impl SafetyGuard {
    pub fn new(config: SafetyGuardConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(
        &self,
        command: OsCommand,
        confidence: f32,
        now: Instant,
        context: &mut SafetyContext,
    ) -> SafetyDecision {
        if matches!(command, OsCommand::NoAction) {
            return SafetyDecision {
                allow: false,
                reason: "Пустая команда.".to_owned(),
                needs_extra_confirmation: false,
            };
        }

        if confidence < self.config.min_confidence {
            return SafetyDecision {
                allow: false,
                reason: format!("Низкая уверенность распознавания ({confidence:.2})."),
                needs_extra_confirmation: false,
            };
        }

        if matches!(command, OsCommand::ShutdownComputer) {
            if confidence < self.config.critical_min_confidence {
                return SafetyDecision {
                    allow: false,
                    reason: "Критичная команда отклонена из-за недостаточной уверенности."
                        .to_owned(),
                    needs_extra_confirmation: true,
                };
            }
            if !context.shutdown_confirmation_pending {
                context.shutdown_confirmation_pending = true;
                return SafetyDecision {
                    allow: false,
                    reason: "Требуется дополнительное подтверждение выключения.".to_owned(),
                    needs_extra_confirmation: true,
                };
            }
        }

        if let Some(prev) = context.last_execution {
            if now.duration_since(prev) < self.config.min_command_interval {
                return SafetyDecision {
                    allow: false,
                    reason: "Антидребезг: команды слишком частые.".to_owned(),
                    needs_extra_confirmation: false,
                };
            }
        }

        context.last_execution = Some(now);
        SafetyDecision {
            allow: true,
            reason: "Команда разрешена.".to_owned(),
            needs_extra_confirmation: false,
        }
    }
}
