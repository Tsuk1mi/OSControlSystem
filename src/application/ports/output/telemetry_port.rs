/// Метрики и диагностика без привязки к UI.
pub trait TelemetryPort: Send {
    fn record_fps(&mut self, fps: f32);
}
