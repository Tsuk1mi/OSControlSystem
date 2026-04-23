use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::context::ContextDetectionMode;
use crate::gesture_os_control::domain::entities::gesture_backend::GestureBackendKind;

const FILE_NAME: &str = "gesture_app_settings.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub selected_video_input: String,
    pub gesture_camera_width: u32,
    pub gesture_camera_height: u32,
    pub gesture_camera_fps: u32,
    pub gesture_mirror_horizontal: bool,
    pub gesture_sensitivity: f32,
    pub gesture_cooldown_secs: f32,
    pub backend_kind: GestureBackendKind,
    #[serde(default)]
    pub mediapipe_model_path: String,
    pub context_detection_mode: ContextDetectionMode,
    pub manual_run_mode: AppRunMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            selected_video_input: "Камера по умолчанию".to_owned(),
            gesture_camera_width: 640,
            gesture_camera_height: 480,
            gesture_camera_fps: 60,
            gesture_mirror_horizontal: false,
            gesture_sensitivity: 0.72,
            gesture_cooldown_secs: 1.2,
            backend_kind: GestureBackendKind::MediaPipe,
            mediapipe_model_path: String::new(),
            context_detection_mode: ContextDetectionMode::Manual,
            manual_run_mode: AppRunMode::Desktop,
        }
    }
}

pub fn settings_path() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    dir.pop();
    Some(dir.join(FILE_NAME))
}

pub fn load_or_default() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str::<AppSettings>(&text).unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or_else(|| "Не удалось определить путь к exe.".to_owned())?;
    save_to_path(settings, &path)
}

pub fn save_to_path(settings: &AppSettings, path: &Path) -> Result<(), String> {
    let text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}
