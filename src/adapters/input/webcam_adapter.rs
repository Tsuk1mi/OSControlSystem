use std::time::{Duration, Instant};

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::{Camera, query};

use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::application::ports::input::camera_input_port::CameraInputPort;

const DEFAULT_DEVICE_LABEL: &str = "Камера по умолчанию";


#[derive(Clone, Debug)]
pub struct WebcamAdapterConfig {
    // Имя устройства из UI или метка по умолчанию.
    pub camera_id: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub mirror_horizontal: bool,
}

impl Default for WebcamAdapterConfig {
    fn default() -> Self {
        Self {
            camera_id: DEFAULT_DEVICE_LABEL.to_owned(),
            width: 640,
            height: 480,
            fps: 30,
            mirror_horizontal: false,
        }
    }
}

// Адаптер веб-камеры: открытие, чтение кадров, проверка доступности, переподключение.
pub struct WebcamAdapter {
    config: WebcamAdapterConfig,
    camera: Option<Camera>,
    frame_index: u64,
    last_frame_instant: Option<Instant>,
    fps_smoothed: f32,
    consecutive_failures: u32,
}

impl WebcamAdapter {
    pub fn new(config: WebcamAdapterConfig) -> Self {
        Self {
            config,
            camera: None,
            frame_index: 0,
            last_frame_instant: None,
            fps_smoothed: 0.0,
            consecutive_failures: 0,
        }
    }

    pub fn fps_smoothed(&self) -> f32 {
        self.fps_smoothed
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    // Проверка доступности выбранной камеры без открытия потока.
    pub fn is_camera_available(camera_id: &str) -> Result<(), String> {
        let cameras = query(ApiBackend::Auto)
            .map_err(|error| format!("Не удалось получить список камер: {error}"))?;
        if cameras.is_empty() {
            return Err("В системе не найдено доступных камер.".to_owned());
        }
        if camera_id == DEFAULT_DEVICE_LABEL {
            return Ok(());
        }
        resolve_camera_index(camera_id, &cameras).map(|_| ())
    }

    pub fn open(&mut self) -> Result<(), String> {
        self.close_internal();
        let cameras = query(ApiBackend::Auto)
            .map_err(|error| format!("Не удалось получить список камер: {error}"))?;
        if cameras.is_empty() {
            return Err("В системе не найдено доступных камер.".to_owned());
        }

        let index = resolve_camera_index(&self.config.camera_id, &cameras)?;
        // Closest в nokhwa сопоставляет только с тем FourCC, что задан в CameraFormat.
        // Запрос «ближайшего MJPEG» падает на камерах без MJPEG. Без фичи decoding у nokhwa
        // MJPEG всё равно не декодируется (нужен mozjpeg). Поэтому сначала YUYV/NV12/RAW.
        let mut camera = open_camera_with_format_fallbacks(index, &self.config)
            .map_err(|error| format!("Не удалось инициализировать камеру: {error}"))?;

        camera
            .open_stream()
            .map_err(|error| format!("Не удалось открыть видеопоток: {error}"))?;

        let _ = camera.set_resolution(Resolution::new(self.config.width, self.config.height));
        let _ = camera.set_frame_rate(self.config.fps);

        self.camera = Some(camera);
        self.frame_index = 0;
        self.last_frame_instant = None;
        self.consecutive_failures = 0;
        Ok(())
    }

    pub fn close(&mut self) {
        self.close_internal();
    }

    fn close_internal(&mut self) {
        if let Some(mut camera) = self.camera.take() {
            let _ = camera.stop_stream();
        }
    }

    // Читает один кадр. При повторяющихся ошибках пытается переподключиться.
    pub fn read_frame(&mut self) -> Result<FrameDto, String> {
        let Some(camera) = self.camera.as_mut() else {
            return Err("Камера не открыта.".to_owned());
        };

        match Self::capture_rgb(camera, self.config.mirror_horizontal) {
            Ok((width, height, rgb8)) => {
                self.consecutive_failures = 0;
                let now = Instant::now();
                if let Some(prev) = self.last_frame_instant.replace(now) {
                    let dt = now.duration_since(prev).as_secs_f32().max(0.000_1);
                    let inst_fps = 1.0 / dt;
                    self.fps_smoothed = if self.fps_smoothed <= 0.0 {
                        inst_fps
                    } else {
                        self.fps_smoothed * 0.85 + inst_fps * 0.15
                    };
                }

                let dto = FrameDto {
                    frame_index: self.frame_index,
                    timestamp: now,
                    width,
                    height,
                    rgb8,
                };
                self.frame_index = self.frame_index.wrapping_add(1);
                Ok(dto)
            }
            Err(error) => {
                self.consecutive_failures = self.consecutive_failures.saturating_add(1);
                if self.consecutive_failures >= 4 {
                    let _ = self.reconnect_after_delay();
                }
                Err(error)
            }
        }
    }

    fn reconnect_after_delay(&mut self) -> Result<(), String> {
        self.close_internal();
        std::thread::sleep(Duration::from_millis(250));
        self.open()
    }

    fn capture_rgb(camera: &mut Camera, mirror_horizontal: bool) -> Result<(u32, u32, Vec<u8>), String> {
        let buffer = camera
            .frame()
            .map_err(|error| format!("Ошибка чтения кадра: {error}"))?;
        let image = buffer
            .decode_image::<RgbFormat>()
            .map_err(|error| format!("Не удалось декодировать кадр: {error}"))?;

        let width = image.width();
        let height = image.height();
        let mut rgb8 = image.into_raw();

        if mirror_horizontal {
            mirror_rgb_in_place(&mut rgb8, width as usize, height as usize);
        }

        Ok((width, height, rgb8))
    }
}

/// Без фичи `decoding` у nokhwa MJPEG не переводится в RGB (нужен mozjpeg), поэтому не приоритизируем его.
fn preferred_uncompressed_formats() -> &'static [FrameFormat] {
    &[
        FrameFormat::YUYV,
        FrameFormat::NV12,
        FrameFormat::RAWRGB,
        FrameFormat::RAWBGR,
    ]
}

fn resolution_candidates(width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for pair in [
        (width, height),
        (640, 480),
        (320, 240),
        (848, 480),
        (1280, 720),
        (1920, 1080),
    ] {
        if pair.0 > 0 && pair.1 > 0 && !out.contains(&pair) {
            out.push(pair);
        }
    }
    out
}

fn open_camera_with_format_fallbacks(
    index: CameraIndex,
    config: &WebcamAdapterConfig,
) -> Result<Camera, nokhwa::NokhwaError> {
    let fps = config.fps.max(1);
    let resolutions = resolution_candidates(config.width, config.height);
    let formats = preferred_uncompressed_formats();
    let mut errors: Vec<nokhwa::NokhwaError> = Vec::new();

    for (rw, rh) in &resolutions {
        for &fourcc in formats {
            let requested = RequestedFormat::with_formats(
                RequestedFormatType::Closest(CameraFormat::new_from(*rw, *rh, fourcc, fps)),
                std::slice::from_ref(&fourcc),
            );
            match Camera::with_backend(index.clone(), requested, ApiBackend::Auto) {
                Ok(cam) => return Ok(cam),
                Err(e) => errors.push(e),
            }
        }
    }

    let requested = RequestedFormat::with_formats(RequestedFormatType::None, formats);
    match Camera::with_backend(index.clone(), requested, ApiBackend::Auto) {
        Ok(cam) => return Ok(cam),
        Err(e) => errors.push(e),
    }

    let requested = RequestedFormat::with_formats(
        RequestedFormatType::AbsoluteHighestResolution,
        formats,
    );
    match Camera::with_backend(index, requested, ApiBackend::Auto) {
        Ok(cam) => return Ok(cam),
        Err(e) => errors.push(e),
    }

    Err(errors.into_iter().next_back().unwrap_or_else(|| {
        nokhwa::NokhwaError::GeneralError("Не удалось подобрать формат камеры.".to_owned())
    }))
}

fn mirror_rgb_in_place(rgb: &mut [u8], width: usize, height: usize) {
    let row_stride = width * 3;
    for y in 0..height {
        let row_start = y * row_stride;
        let row = &mut rgb[row_start..row_start + row_stride];
        for x in 0..width / 2 {
            let a = x * 3;
            let b = (width - 1 - x) * 3;
            for i in 0..3 {
                row.swap(a + i, b + i);
            }
        }
    }
}

fn resolve_camera_index(camera_id: &str, cameras: &[nokhwa::utils::CameraInfo]) -> Result<CameraIndex, String> {
    if camera_id == DEFAULT_DEVICE_LABEL {
        return Ok(cameras[0].index().clone());
    }

    let normalized = normalize_device_name(camera_id);
    cameras
        .iter()
        .find(|camera| normalize_device_name(&camera.human_name()) == normalized)
        .or_else(|| {
            cameras.iter().find(|camera| {
                normalize_device_name(&camera.human_name()).contains(&normalized)
            })
        })
        .map(|camera| camera.index().clone())
        .ok_or_else(|| format!("Камера `{camera_id}` не найдена."))
}

fn normalize_device_name(value: &str) -> String {
    value
        .to_lowercase()
        .replace('ё', "е")
        .chars()
        .map(|char| {
            if char.is_alphanumeric() || char.is_whitespace() {
                char
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl CameraInputPort for WebcamAdapter {
    fn pull_frame(&mut self) -> Result<FrameDto, String> {
        self.read_frame()
    }

    fn release_input(&mut self) {
        self.close();
    }
}
