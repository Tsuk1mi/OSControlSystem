//! Заглушка шины событий между слоями.

#[derive(Default)]
pub struct EventBus;

impl EventBus {
    pub fn new() -> Self {
        Self
    }
}
