use crate::gesture_os_control::application::dto::frame_dto::FrameDto;

/// Порт захвата кадров (камера, файл, тестовый поток).
pub trait CameraInputPort {
    fn pull_frame(&mut self) -> Result<FrameDto, String>;
    fn release_input(&mut self);
}
