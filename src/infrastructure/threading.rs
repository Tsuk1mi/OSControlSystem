//! Вспомогательные примитивы потоков (пока не используются отдельно от сервисов приложения).

#[derive(Default)]
pub struct ThreadingHints {
    pub gesture_worker_stack_kb: u32,
}

impl ThreadingHints {
    pub fn desktop_default() -> Self {
        Self {
            gesture_worker_stack_kb: 2048,
        }
    }
}
