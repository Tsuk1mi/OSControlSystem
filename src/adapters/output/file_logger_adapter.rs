use crate::gesture_os_control::application::ports::output::log_port::LogPort;

/// Простейший адаптер журнала в память (для тестов и отладки).
#[derive(Default)]
pub struct FileLoggerAdapter {
    lines: Vec<String>,
}

impl FileLoggerAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl LogPort for FileLoggerAdapter {
    fn info(&mut self, message: &str) {
        self.lines.push(format!("[INFO] {message}"));
    }

    fn warn(&mut self, message: &str) {
        self.lines.push(format!("[WARN] {message}"));
    }
}
