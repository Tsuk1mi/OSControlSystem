use crate::gesture_os_control::application::ports::input::ui_input_port::UiInputPort;

/// Заглушка UI-порта (события пока не прокидываются в домен).
pub struct UiControllerAdapter;

impl UiInputPort for UiControllerAdapter {
    fn poll_pending_action(&mut self) -> Option<String> {
        None
    }
}
