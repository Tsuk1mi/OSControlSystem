use crate::gesture_os_control::application::dto::frame_dto::FrameDto;

/// Тестовый ввод кадров без устройства.
pub trait TestInputPort {
    fn next_synthetic_frame(&mut self) -> Option<FrameDto>;
}
