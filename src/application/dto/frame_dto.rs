#![allow(dead_code)]

use std::time::Instant;

/// Кадр с веб-камеры в нормализованном виде для конвейера.
#[derive(Clone, Debug)]
pub struct FrameDto {
    pub frame_index: u64,
    pub timestamp: Instant,
    pub width: u32,
    pub height: u32,
    /// RGB888, длина `width * height * 3`.
    pub rgb8: Vec<u8>,
}
