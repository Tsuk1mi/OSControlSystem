/// События и команды из UI (заглушка под будущую интеграцию).
pub trait UiInputPort: Send {
    fn poll_pending_action(&mut self) -> Option<String>;
}
