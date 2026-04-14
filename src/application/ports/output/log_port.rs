/// Структурированный журнал конвейера жестов.
pub trait LogPort: Send {
    fn info(&mut self, message: &str);
    fn warn(&mut self, message: &str);
}
