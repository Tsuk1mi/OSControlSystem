use serde::{Deserialize, Serialize};

use crate::gesture_os_control::domain::entities::gesture::AppRunMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContextDetectionMode {
    #[default]
    Manual,
    Auto,
}

impl ContextDetectionMode {
    pub fn label_ru(self) -> &'static str {
        match self {
            Self::Manual => "ручной",
            Self::Auto => "авто",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContextDecisionSource {
    #[default]
    Manual,
    Rule,
    Fallback,
}

impl ContextDecisionSource {
    pub fn label_ru(self) -> &'static str {
        match self {
            Self::Manual => "вручную",
            Self::Rule => "по правилу",
            Self::Fallback => "fallback",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextRule {
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub process_name_contains: String,
    #[serde(default)]
    pub window_title_contains: String,
    pub mode: AppRunMode,
}

impl ContextRule {
    pub fn new(
        name: impl Into<String>,
        process_name_contains: impl Into<String>,
        window_title_contains: impl Into<String>,
        mode: AppRunMode,
    ) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            process_name_contains: process_name_contains.into(),
            window_title_contains: window_title_contains.into(),
            mode,
        }
    }

    pub fn matches(&self, foreground: &ForegroundWindowInfo) -> bool {
        if !self.enabled {
            return false;
        }
        let process_ok =
            contains_case_insensitive(&foreground.process_name, self.process_name_contains.trim());
        let title_ok =
            contains_case_insensitive(&foreground.window_title, self.window_title_contains.trim());
        process_ok && title_ok
    }
}

#[derive(Clone, Debug, Default)]
pub struct ForegroundWindowInfo {
    pub process_name: String,
    pub window_title: String,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedAppContext {
    pub mode: AppRunMode,
    pub source: ContextDecisionSource,
    pub process_name: String,
    pub window_title: String,
    pub matched_rule_name: Option<String>,
}

impl ResolvedAppContext {
    pub fn summary_ru(&self) -> String {
        let mut line = format!("{} ({})", self.mode.label_ru(), self.source.label_ru());
        if let Some(rule) = &self.matched_rule_name {
            if !rule.is_empty() {
                line.push_str(&format!(" · правило `{rule}`"));
            }
        }
        if !self.process_name.is_empty() {
            line.push_str(&format!(" · {}", self.process_name));
        }
        if !self.window_title.is_empty() {
            line.push_str(&format!(" · {}", self.window_title));
        }
        line
    }
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}
