use crate::gesture_os_control::application::ports::output::config_port::ConfigPort;

/// Заглушка TOML-конфига: значения в памяти до появления файла настроек.
pub struct TomlConfigAdapter {
    gesture_sensitivity: f32,
}

impl Default for TomlConfigAdapter {
    fn default() -> Self {
        Self {
            gesture_sensitivity: 0.72,
        }
    }
}

impl TomlConfigAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConfigPort for TomlConfigAdapter {
    fn gesture_sensitivity(&self) -> f32 {
        self.gesture_sensitivity
    }

    fn set_gesture_sensitivity(&mut self, value: f32) {
        self.gesture_sensitivity = value.clamp(0.0, 1.0);
    }
}
