use crate::gesture_os_control::domain::entities::context::{
    ContextDecisionSource, ContextDetectionMode, ContextRule, ForegroundWindowInfo,
    ResolvedAppContext,
};
use crate::gesture_os_control::domain::entities::gesture::AppRunMode;

pub struct ContextResolver;

impl ContextResolver {
    pub fn resolve(
        detection_mode: ContextDetectionMode,
        manual_mode: AppRunMode,
        rules: &[ContextRule],
        foreground: Option<&ForegroundWindowInfo>,
    ) -> ResolvedAppContext {
        if detection_mode == ContextDetectionMode::Manual {
            return Self::manual(manual_mode, foreground);
        }

        if let Some(foreground) = foreground {
            if let Some(rule) = rules.iter().find(|rule| rule.matches(foreground)) {
                return ResolvedAppContext {
                    mode: rule.mode,
                    source: ContextDecisionSource::Rule,
                    process_name: foreground.process_name.clone(),
                    window_title: foreground.window_title.clone(),
                    matched_rule_name: Some(rule.name.clone()),
                };
            }

            return ResolvedAppContext {
                mode: AppRunMode::Desktop,
                source: ContextDecisionSource::Fallback,
                process_name: foreground.process_name.clone(),
                window_title: foreground.window_title.clone(),
                matched_rule_name: None,
            };
        }

        ResolvedAppContext {
            mode: AppRunMode::Desktop,
            source: ContextDecisionSource::Fallback,
            process_name: String::new(),
            window_title: String::new(),
            matched_rule_name: None,
        }
    }

    fn manual(
        manual_mode: AppRunMode,
        foreground: Option<&ForegroundWindowInfo>,
    ) -> ResolvedAppContext {
        let (process_name, window_title) = foreground
            .map(|info| (info.process_name.clone(), info.window_title.clone()))
            .unwrap_or_default();
        ResolvedAppContext {
            mode: manual_mode,
            source: ContextDecisionSource::Manual,
            process_name,
            window_title,
            matched_rule_name: None,
        }
    }
}
