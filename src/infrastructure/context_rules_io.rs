use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::context::ContextRule;

const FILE_NAME: &str = "context_rules.json";

#[derive(Serialize, Deserialize, Default)]
struct ContextRulesFile {
    #[serde(default)]
    rules: Vec<ContextRule>,
}

pub fn rules_path() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    dir.pop();
    Some(dir.join(FILE_NAME))
}

pub fn load_or_defaults() -> Vec<ContextRule> {
    let Some(path) = rules_path() else {
        return default_rules();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return default_rules();
    };
    let Ok(file) = serde_json::from_str::<ContextRulesFile>(&text) else {
        return default_rules();
    };
    if file.rules.is_empty() {
        default_rules()
    } else {
        file.rules
    }
}

pub fn save(rules: &[ContextRule]) -> Result<(), String> {
    let path = rules_path().ok_or_else(|| "Не удалось определить путь к exe.".to_owned())?;
    save_to_path(rules, &path)
}

pub fn save_to_path(rules: &[ContextRule], path: &Path) -> Result<(), String> {
    let file = ContextRulesFile {
        rules: rules.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}

fn default_rules() -> Vec<ContextRule> {
    vec![
        ContextRule::new("Chrome / Edge / Firefox", "chrome", "", AppRunMode::Browser),
        ContextRule::new("Microsoft Edge", "msedge", "", AppRunMode::Browser),
        ContextRule::new("Firefox", "firefox", "", AppRunMode::Browser),
        ContextRule::new("Opera", "opera", "", AppRunMode::Browser),
        ContextRule::new("Spotify", "spotify", "", AppRunMode::Media),
        ContextRule::new("VLC", "vlc", "", AppRunMode::Media),
        ContextRule::new("YouTube вкладка", "", "youtube", AppRunMode::Media),
        ContextRule::new("Netflix вкладка", "", "netflix", AppRunMode::Media),
    ]
}
