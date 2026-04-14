use std::collections::VecDeque;
use std::time::Instant;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

use crate::gesture_os_control::adapters::input::video_device_query::list_video_inputs;
use crate::gesture_os_control::domain::services::command_mapper::GestureCommandMap;
use crate::gesture_os_control::infrastructure::gesture_bindings_io;
use crate::gesture_os_control::infrastructure::gesture_camera_service::{
    GestureCameraConfig, GestureRecognitionService, GestureServiceMessage, WebcamGestureService,
};
use crate::gesture_os_control::{AppRunMode, PipelineGestureStats};

const LOG_CAP: usize = 300;

fn ensure_non_empty(mut items: Vec<String>, fallback: &str) -> Vec<String> {
    if items.is_empty() {
        items.push(fallback.to_owned());
    }
    items
}

pub struct GestureViewModel {
    gesture_command_map: GestureCommandMap,
    gesture_service: WebcamGestureService,
    available_video_inputs: Vec<String>,
    selected_video_input: String,
    gesture_camera_width: u32,
    gesture_camera_height: u32,
    gesture_camera_fps: u32,
    gesture_mirror_horizontal: bool,
    gesture_run_mode: AppRunMode,
    gesture_sensitivity: f32,
    /// Пауза без распознавания после срабатывания жеста (секунды).
    gesture_cooldown_secs: f32,
    gesture_camera_status: String,
    gesture_backend_name: String,
    gesture_pipeline_stats: Option<PipelineGestureStats>,
    started: Instant,
    event_log: VecDeque<String>,
    preview_pending: Option<(u32, u32, Vec<u8>)>,
    preview_texture: Option<TextureHandle>,
}

impl GestureViewModel {
    pub fn new() -> Self {
        let gesture_service = WebcamGestureService::default();
        let gesture_backend_name = gesture_service.backend_name().to_owned();
        let available_video_inputs = ensure_non_empty(list_video_inputs(), "Камера по умолчанию");
        let selected_video_input = available_video_inputs
            .first()
            .cloned()
            .unwrap_or_else(|| "Камера по умолчанию".to_owned());

        let gesture_command_map = gesture_bindings_io::load_merged_with_defaults();
        let mut vm = Self {
            gesture_command_map,
            gesture_service,
            available_video_inputs,
            selected_video_input,
            gesture_camera_width: 640,
            gesture_camera_height: 480,
            gesture_camera_fps: 30,
            gesture_mirror_horizontal: false,
            gesture_run_mode: AppRunMode::Desktop,
            gesture_sensitivity: 0.72,
            gesture_cooldown_secs: 1.2,
            gesture_camera_status: "—".to_owned(),
            gesture_backend_name,
            gesture_pipeline_stats: None,
            started: Instant::now(),
            event_log: VecDeque::new(),
            preview_pending: None,
            preview_texture: None,
        };
        vm.push_log("Готово.");
        vm
    }

    pub fn push_log_line(&mut self, line: impl Into<String>) {
        self.push_log(line);
    }

    fn push_log(&mut self, line: impl Into<String>) {
        let t = self.started.elapsed().as_secs_f32();
        self.event_log.push_back(format!("{:>6.1}s  {}", t, line.into()));
        while self.event_log.len() > LOG_CAP {
            self.event_log.pop_front();
        }
    }

    pub fn title(&self) -> &'static str {
        "Жесты"
    }

    pub fn sync_preview_texture(&mut self, ctx: &egui::Context) {
        if let Some((w, h, buf)) = self.preview_pending.take() {
            let wu = w as usize;
            let hu = h as usize;
            if wu > 0 && hu > 0 && buf.len() >= wu * hu * 3 {
                let color_image = ColorImage::from_rgb([wu, hu], &buf[..wu * hu * 3]);
                match &mut self.preview_texture {
                    Some(tex) => tex.set(color_image, TextureOptions::LINEAR),
                    None => {
                        self.preview_texture = Some(
                            ctx.load_texture("webcam_preview", color_image, TextureOptions::LINEAR),
                        );
                    }
                }
            }
        }
    }

    pub fn preview_texture(&self) -> Option<&TextureHandle> {
        self.preview_texture.as_ref()
    }

    pub fn event_log(&self) -> &VecDeque<String> {
        &self.event_log
    }

    pub fn tick(&mut self) {
        for message in self.gesture_service.poll_messages() {
            match message {
                GestureServiceMessage::Status(status) => {
                    self.gesture_camera_status = status.clone();
                    self.push_log(status);
                }
                GestureServiceMessage::PipelineStats(stats) => {
                    self.gesture_pipeline_stats = Some(stats);
                }
                GestureServiceMessage::DeviceError(error) => {
                    self.gesture_camera_status = error.clone();
                    self.push_log(format!("ошибка: {error}"));
                }
                GestureServiceMessage::PreviewFrame { width, height, rgb8 } => {
                    self.preview_pending = Some((width, height, rgb8));
                }
                GestureServiceMessage::GestureLog(line) => {
                    self.push_log(line);
                }
            }
        }
    }

    pub fn gesture_command_map(&self) -> &GestureCommandMap {
        &self.gesture_command_map
    }

    pub fn gesture_command_map_mut(&mut self) -> &mut GestureCommandMap {
        &mut self.gesture_command_map
    }

    pub fn save_gesture_bindings_to_file(&mut self) -> Result<(), String> {
        gesture_bindings_io::save(&self.gesture_command_map)?;
        self.push_log("Привязки сохранены: gesture_bindings.json рядом с exe.");
        if self.is_gesture_camera_running() {
            self.push_log("Перезапустите камеру (Стоп → Старт), чтобы применить привязки.");
        }
        Ok(())
    }

    pub fn start_gesture_camera(&mut self) {
        let camera = GestureCameraConfig {
            device_display_name: self.selected_video_input.clone(),
            width: self.gesture_camera_width,
            height: self.gesture_camera_height,
            fps: self.gesture_camera_fps,
            mirror_horizontal: self.gesture_mirror_horizontal,
            run_mode: self.gesture_run_mode,
            command_map: self.gesture_command_map.clone(),
            gesture_cooldown_ms: (self.gesture_cooldown_secs * 1000.0).round() as u32,
        };

        match self
            .gesture_service
            .start(self.gesture_sensitivity, &camera)
        {
            Ok(status) => {
                self.gesture_camera_status = status.clone();
                self.push_log(status);
            }
            Err(error) => {
                self.gesture_camera_status = error.clone();
                self.push_log(error);
            }
        }
    }

    pub fn stop_gesture_camera(&mut self) {
        self.preview_pending = None;
        self.preview_texture = None;
        match self.gesture_service.stop() {
            Ok(status) => {
                self.gesture_camera_status = status.clone();
                self.push_log(status);
            }
            Err(error) => {
                self.gesture_camera_status = error.clone();
                self.push_log(error);
            }
        }
    }

    pub fn gesture_camera_status(&self) -> &str {
        &self.gesture_camera_status
    }

    pub fn gesture_backend_name(&self) -> &str {
        &self.gesture_backend_name
    }

    pub fn is_gesture_camera_running(&self) -> bool {
        self.gesture_service.is_running()
    }

    pub fn available_video_inputs(&self) -> &[String] {
        &self.available_video_inputs
    }

    pub fn selected_video_input(&self) -> &str {
        &self.selected_video_input
    }

    pub fn set_selected_video_input(&mut self, device_name: &str) {
        self.selected_video_input = device_name.to_owned();
        self.push_log(format!("камера: {device_name}"));

        if self.gesture_service.is_running() {
            let _ = self.gesture_service.stop();
            self.start_gesture_camera();
        }
    }

    pub fn gesture_pipeline_stats(&self) -> Option<&PipelineGestureStats> {
        self.gesture_pipeline_stats.as_ref()
    }

    pub fn gesture_mirror_horizontal(&self) -> bool {
        self.gesture_mirror_horizontal
    }

    pub fn set_gesture_mirror_horizontal(&mut self, mirror: bool) {
        self.gesture_mirror_horizontal = mirror;
        self.push_log(if mirror { "зеркало: да" } else { "зеркало: нет" });
    }

    pub fn gesture_camera_resolution(&self) -> (u32, u32) {
        (self.gesture_camera_width, self.gesture_camera_height)
    }

    pub fn set_gesture_camera_resolution(&mut self, width: u32, height: u32) {
        self.gesture_camera_width = width.max(160);
        self.gesture_camera_height = height.max(120);
        self.push_log(format!("{}×{}", self.gesture_camera_width, self.gesture_camera_height));
    }

    pub fn gesture_camera_fps(&self) -> u32 {
        self.gesture_camera_fps
    }

    pub fn set_gesture_camera_fps(&mut self, fps: u32) {
        self.gesture_camera_fps = fps.clamp(5, 60);
        self.push_log(format!("fps {}", self.gesture_camera_fps));
    }

    pub fn gesture_run_mode(&self) -> AppRunMode {
        self.gesture_run_mode
    }

    pub fn set_gesture_run_mode(&mut self, mode: AppRunMode) {
        self.gesture_run_mode = mode;
        let s = match mode {
            AppRunMode::Desktop => "контекст: стол",
            AppRunMode::Media => "контекст: медиа",
            AppRunMode::Browser => "контекст: браузер",
        };
        self.push_log(s);
    }

    pub fn gesture_sensitivity(&self) -> f32 {
        self.gesture_sensitivity
    }

    pub fn set_gesture_sensitivity(&mut self, value: f32) {
        self.gesture_sensitivity = value.clamp(0.1, 1.0);
        self.gesture_service.set_sensitivity(self.gesture_sensitivity);
        self.push_log(format!("чувств. {:.0}%", self.gesture_sensitivity * 100.0));
    }

    pub fn gesture_cooldown_secs(&self) -> f32 {
        self.gesture_cooldown_secs
    }

    pub fn set_gesture_cooldown_secs(&mut self, secs: f32) {
        self.gesture_cooldown_secs = secs.clamp(0.0, 10.0);
        let ms = (self.gesture_cooldown_secs * 1000.0).round() as u32;
        self.gesture_service.set_gesture_cooldown_ms(ms);
        self.push_log(format!("пауза после жеста: {:.1} с", self.gesture_cooldown_secs));
    }
}
