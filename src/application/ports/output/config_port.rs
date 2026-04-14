/// Чтение/запись настроек приложения (файл, реестр и т.д.).
pub trait ConfigPort: Send {
    fn gesture_sensitivity(&self) -> f32;
    fn set_gesture_sensitivity(&mut self, value: f32);
}
