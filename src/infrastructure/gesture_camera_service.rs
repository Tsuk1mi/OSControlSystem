use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::gesture_os_control::adapters::input::webcam_adapter::{
    WebcamAdapter, WebcamAdapterConfig,
};
use crate::gesture_os_control::adapters::input::windows_foreground_window_adapter::read_foreground_window;
#[cfg(not(windows))]
use crate::gesture_os_control::adapters::output::linux_os_adapter::LinuxOsAdapter;
#[cfg(windows)]
use crate::gesture_os_control::adapters::output::windows_os_adapter::WindowsPipelineOsAdapter;
use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::application::dto::gesture_debug_dto::GestureDebugFrameDto;
use crate::gesture_os_control::application::use_cases::execute_command_use_case::ExecuteCommandUseCase;
use crate::gesture_os_control::application::use_cases::process_frame_use_case::ProcessFrameUseCase;
use crate::gesture_os_control::application::use_cases::recognize_gesture_use_case::{
    RecognizeGestureUseCase, RecognizedFrame,
};
use crate::gesture_os_control::domain::entities::context::{
    ContextDetectionMode, ContextRule, ForegroundWindowInfo,
};
use crate::gesture_os_control::domain::entities::gesture::{
    AppRunMode, FrameProcessingOutcome, PipelineGestureStats,
};
use crate::gesture_os_control::domain::entities::session_state::FrameProcessingSession;
use crate::gesture_os_control::domain::services::command_mapper::GestureCommandMap;
use crate::gesture_os_control::domain::services::context_resolver::ContextResolver;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;
use crate::gesture_os_control::infrastructure::gesture_backend::{
    GestureBackendConfig, create_backend,
};

const CAMERA_BACKEND_NAME: &str = "Жесты";

/// Параметры видеопотока и режима обработки жестов.
#[derive(Clone, Debug)]
pub struct GestureCameraConfig {
    pub device_display_name: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub mirror_horizontal: bool,
    pub manual_run_mode: AppRunMode,
    pub context_detection_mode: ContextDetectionMode,
    pub context_rules: Vec<ContextRule>,
    pub backend: GestureBackendConfig,
    pub command_map: GestureCommandMap,
    /// Пауза без распознавания после успешной команды (антиспам), миллисекунды.
    pub gesture_cooldown_ms: u32,
}

impl Default for GestureCameraConfig {
    fn default() -> Self {
        Self {
            device_display_name: "Камера по умолчанию".to_owned(),
            width: 640,
            height: 480,
            fps: 60,
            mirror_horizontal: false,
            manual_run_mode: AppRunMode::Desktop,
            context_detection_mode: ContextDetectionMode::Manual,
            context_rules: Vec::new(),
            backend: GestureBackendConfig::default(),
            command_map: GestureCommandMap::app_defaults(),
            gesture_cooldown_ms: 1200,
        }
    }
}

pub enum GestureServiceMessage {
    Status(String),
    PipelineStats(PipelineGestureStats),
    DeviceError(String),
    /// Уменьшенный кадр RGB888 для превью в UI.
    PreviewFrame {
        width: u32,
        height: u32,
        rgb8: Vec<u8>,
    },
    DebugFrame(GestureDebugFrameDto),
    /// Строка журнала распознавания (жест, фильтр, команда).
    GestureLog(String),
}

pub trait GestureRecognitionService {
    fn backend_name(&self) -> &'static str;
    fn start(&mut self, sensitivity: f32, camera: &GestureCameraConfig) -> Result<String, String>;
    fn stop(&mut self) -> Result<String, String>;
    fn is_running(&self) -> bool;
    fn set_sensitivity(&mut self, sensitivity: f32);
    fn set_gesture_cooldown_ms(&mut self, ms: u32);
    fn set_context_detection_mode(&mut self, mode: ContextDetectionMode);
    fn set_manual_run_mode(&mut self, mode: AppRunMode);
    fn set_context_rules(&mut self, rules: &[ContextRule]);
    fn poll_messages(&mut self) -> Vec<GestureServiceMessage>;
}

#[derive(Clone, Debug)]
struct LiveContextConfig {
    detection_mode: ContextDetectionMode,
    manual_run_mode: AppRunMode,
    rules: Vec<ContextRule>,
}

pub struct WebcamGestureService {
    sensitivity: f32,
    sensitivity_live: Option<Arc<AtomicU32>>,
    gesture_cooldown_live: Option<Arc<AtomicU32>>,
    context_live: Option<Arc<RwLock<LiveContextConfig>>>,
    running: bool,
    receiver: Option<Receiver<GestureServiceMessage>>,
    stop_sender: Option<Sender<()>>,
    worker_handle: Option<JoinHandle<()>>,
    pending_messages: Vec<GestureServiceMessage>,
}

impl Default for WebcamGestureService {
    fn default() -> Self {
        Self {
            sensitivity: 0.72,
            sensitivity_live: None,
            gesture_cooldown_live: None,
            context_live: None,
            running: false,
            receiver: None,
            stop_sender: None,
            worker_handle: None,
            pending_messages: Vec::new(),
        }
    }
}

impl GestureRecognitionService for WebcamGestureService {
    fn backend_name(&self) -> &'static str {
        CAMERA_BACKEND_NAME
    }

    fn start(&mut self, sensitivity: f32, camera: &GestureCameraConfig) -> Result<String, String> {
        if self.running {
            return Ok("Уже запущена.".to_owned());
        }

        WebcamAdapter::is_camera_available(&camera.device_display_name)?;

        self.sensitivity = sensitivity;
        let sen_arc = Arc::new(AtomicU32::new(sensitivity.to_bits()));
        self.sensitivity_live = Some(sen_arc.clone());

        let cd_ms = camera.gesture_cooldown_ms.clamp(0, 600_000);
        let cd_arc = Arc::new(AtomicU32::new(cd_ms));
        self.gesture_cooldown_live = Some(cd_arc.clone());

        let live_context = Arc::new(RwLock::new(LiveContextConfig {
            detection_mode: camera.context_detection_mode,
            manual_run_mode: camera.manual_run_mode,
            rules: camera.context_rules.clone(),
        }));
        self.context_live = Some(live_context.clone());

        let (message_tx, message_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let camera = camera.clone();
        let status_device = camera.device_display_name.clone();

        let handle = thread::spawn(move || {
            if let Err(error) = pipeline_worker(
                sen_arc,
                cd_arc,
                live_context,
                camera,
                message_tx.clone(),
                stop_rx,
            ) {
                let _ = message_tx.send(GestureServiceMessage::DeviceError(error));
            }
        });

        self.receiver = Some(message_rx);
        self.stop_sender = Some(stop_tx);
        self.worker_handle = Some(handle);
        self.running = true;

        Ok(format!("Камера `{status_device}`."))
    }

    fn stop(&mut self) -> Result<String, String> {
        if !self.running {
            return Ok("Уже остановлена.".to_owned());
        }

        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            handle
                .join()
                .map_err(|_| "Поток камеры аварийно завершился.".to_owned())?;
        }

        self.receiver = None;
        self.running = false;
        self.sensitivity_live = None;
        self.gesture_cooldown_live = None;
        self.context_live = None;

        Ok("Стоп.".to_owned())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
        if let Some(arc) = &self.sensitivity_live {
            arc.store(sensitivity.to_bits(), Ordering::Relaxed);
        }
    }

    fn set_gesture_cooldown_ms(&mut self, ms: u32) {
        if let Some(arc) = &self.gesture_cooldown_live {
            arc.store(ms.clamp(0, 600_000), Ordering::Relaxed);
        }
    }

    fn set_context_detection_mode(&mut self, mode: ContextDetectionMode) {
        if let Some(state) = &self.context_live {
            if let Ok(mut guard) = state.write() {
                guard.detection_mode = mode;
            }
        }
    }

    fn set_manual_run_mode(&mut self, mode: AppRunMode) {
        if let Some(state) = &self.context_live {
            if let Ok(mut guard) = state.write() {
                guard.manual_run_mode = mode;
            }
        }
    }

    fn set_context_rules(&mut self, rules: &[ContextRule]) {
        if let Some(state) = &self.context_live {
            if let Ok(mut guard) = state.write() {
                guard.rules = rules.to_vec();
            }
        }
    }

    fn poll_messages(&mut self) -> Vec<GestureServiceMessage> {
        let mut messages = std::mem::take(&mut self.pending_messages);

        if let Some(receiver) = &self.receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        messages
    }
}

impl Drop for WebcamGestureService {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn rgb_preview_downsample(frame: &FrameDto, max_side: u32) -> (u32, u32, Vec<u8>) {
    let fw = frame.width.max(1);
    let fh = frame.height.max(1);
    let scale = (max_side as f32 / fw.max(fh) as f32).min(1.0);
    let nw = ((fw as f32 * scale).round() as u32).max(1);
    let nh = ((fh as f32 * scale).round() as u32).max(1);
    let mut out = vec![0u8; (nw * nh * 3) as usize];
    for ty in 0..nh {
        for tx in 0..nw {
            let sx = (tx * fw) / nw;
            let sy = (ty * fh) / nh;
            let si = ((sy * fw + sx) * 3) as usize;
            let di = ((ty * nw + tx) * 3) as usize;
            if si + 3 <= frame.rgb8.len() && di + 3 <= out.len() {
                out[di..di + 3].copy_from_slice(&frame.rgb8[si..si + 3]);
            }
        }
    }
    (nw, nh, out)
}

fn pipeline_worker(
    sensitivity_bits: Arc<AtomicU32>,
    gesture_cooldown_ms: Arc<AtomicU32>,
    context_live: Arc<RwLock<LiveContextConfig>>,
    camera: GestureCameraConfig,
    message_tx: Sender<GestureServiceMessage>,
    stop_rx: Receiver<()>,
) -> Result<(), String> {
    let adapter_config = WebcamAdapterConfig {
        camera_id: camera.device_display_name.clone(),
        width: camera.width,
        height: camera.height,
        fps: camera.fps,
        mirror_horizontal: camera.mirror_horizontal,
    };

    let mut adapter = WebcamAdapter::new(adapter_config);
    adapter.open()?;

    let _ = message_tx.send(GestureServiceMessage::Status("Камера открыта.".to_owned()));

    let mut session = FrameProcessingSession::with_command_map(camera.command_map.clone());
    let mut backend = create_backend(&camera.backend);
    let mut last_stats_sent = Instant::now();
    let mut last_gesture_log = Instant::now();
    let mut last_foreground_poll = Instant::now() - Duration::from_secs(1);
    let mut cached_foreground: Option<ForegroundWindowInfo> = None;
    let mut last_context_line = String::new();

    loop {
        if stop_rx.try_recv().is_ok() {
            let _ = message_tx.send(GestureServiceMessage::Status(
                "Камера остановлена.".to_owned(),
            ));
            adapter.close();
            return Ok(());
        }

        match adapter.read_frame() {
            Ok(frame) => {
                if frame.frame_index % 4 == 0 {
                    let (pw, ph, buf) = rgb_preview_downsample(&frame, 480);
                    let _ = message_tx.send(GestureServiceMessage::PreviewFrame {
                        width: pw,
                        height: ph,
                        rgb8: buf,
                    });
                }

                let sensitivity = f32::from_bits(sensitivity_bits.load(Ordering::Relaxed));
                let cd_ms = gesture_cooldown_ms.load(Ordering::Relaxed);
                let gesture_cooldown = Duration::from_millis(cd_ms as u64);
                if last_foreground_poll.elapsed() >= Duration::from_millis(250) {
                    cached_foreground = read_foreground_window();
                    last_foreground_poll = Instant::now();
                }
                let context_cfg =
                    context_live
                        .read()
                        .map(|guard| guard.clone())
                        .unwrap_or(LiveContextConfig {
                            detection_mode: camera.context_detection_mode,
                            manual_run_mode: camera.manual_run_mode,
                            rules: camera.context_rules.clone(),
                        });
                let resolved_context = ContextResolver::resolve(
                    context_cfg.detection_mode,
                    context_cfg.manual_run_mode,
                    &context_cfg.rules,
                    cached_foreground.as_ref(),
                );
                let recognized = match RecognizeGestureUseCase::execute(
                    &mut session,
                    backend.as_mut(),
                    &frame,
                    sensitivity,
                ) {
                    Ok(recognized) => recognized,
                    Err(error) => {
                        let _ = message_tx
                            .send(GestureServiceMessage::Status(format!("backend: {error}")));
                        RecognizedFrame {
                            raw_gesture: RecognizeGestureUseCase::empty_raw(&frame),
                            debug_frame: GestureDebugFrameDto {
                                frame_width: frame.width,
                                frame_height: frame.height,
                                backend_name: backend.backend_name().to_owned(),
                                ..GestureDebugFrameDto::default()
                            },
                        }
                    }
                };
                let result = ProcessFrameUseCase::execute(
                    resolved_context.mode,
                    gesture_cooldown,
                    &mut session,
                    recognized.clone(),
                );
                session.stats.fps_smoothed = adapter.fps_smoothed();
                session.stats.consecutive_errors = adapter.consecutive_failures();
                let mut debug_frame = recognized.debug_frame;
                debug_frame.filter_stability = result.filter_stability;
                debug_frame.filter_status = result.filter_status;
                debug_frame.filter_reason = result.filter_reason.clone();
                debug_frame.resolved_mode = resolved_context.mode;
                debug_frame.context_summary = resolved_context.summary_ru();
                if frame.frame_index % 4 == 0 {
                    let _ = message_tx.send(GestureServiceMessage::DebugFrame(debug_frame));
                }
                let context_line = resolved_context.summary_ru();
                if context_line != last_context_line {
                    last_context_line = context_line.clone();
                    let _ = message_tx.send(GestureServiceMessage::Status(format!(
                        "контекст: {context_line}"
                    )));
                }

                if last_stats_sent.elapsed() >= Duration::from_millis(180) {
                    last_stats_sent = Instant::now();
                    let _ = message_tx
                        .send(GestureServiceMessage::PipelineStats(session.stats.clone()));
                }

                if last_gesture_log.elapsed() >= Duration::from_millis(280) {
                    last_gesture_log = Instant::now();
                    let g = result.raw_gesture.gesture;
                    if g != GestureId::None {
                        let name = g.user_trigger_ru().unwrap_or("?");
                        let _ = message_tx.send(GestureServiceMessage::GestureLog(format!(
                            "{} · {:.0}% · устойч. {:.0}% · {} · {}",
                            name,
                            result.raw_gesture.confidence * 100.0,
                            result.filter_stability * 100.0,
                            resolved_context.mode.label_ru(),
                            result.filter_status.label_ru(),
                        )));
                    }
                }

                match result.outcome {
                    FrameProcessingOutcome::CommandReady { command } => {
                        let trigger = session
                            .stats
                            .last_gesture
                            .user_trigger_ru()
                            .unwrap_or("жест")
                            .to_owned();
                        let msg_tx = message_tx.clone();
                        #[cfg(windows)]
                        thread::spawn(move || {
                            let port = WindowsPipelineOsAdapter::new();
                            let exec = ExecuteCommandUseCase::run(&port, command);
                            let line = if exec.ok {
                                format!(
                                    "OK {} · {trigger} · {}",
                                    command.label_ru(),
                                    resolved_context.mode.label_ru()
                                )
                            } else {
                                format!(
                                    "нет {}: {}",
                                    command.label_ru(),
                                    exec.system_error.unwrap_or_default()
                                )
                            };
                            let _ = msg_tx.send(GestureServiceMessage::Status(line));
                        });
                        #[cfg(not(windows))]
                        thread::spawn(move || {
                            let port = LinuxOsAdapter::new();
                            let exec = ExecuteCommandUseCase::run(&port, command);
                            let line = if exec.ok {
                                format!(
                                    "OK {} · {trigger} · {}",
                                    command.label_ru(),
                                    resolved_context.mode.label_ru()
                                )
                            } else {
                                format!(
                                    "нет {}: {}",
                                    command.label_ru(),
                                    exec.system_error.unwrap_or_default()
                                )
                            };
                            let _ = msg_tx.send(GestureServiceMessage::Status(line));
                        });
                    }
                    FrameProcessingOutcome::GestureConfirmedCommandDenied { reason } => {
                        let _ = message_tx
                            .send(GestureServiceMessage::Status(format!("защита: {reason}")));
                    }
                    FrameProcessingOutcome::GestureRejected { .. } => {}
                    _ => {}
                }
            }
            Err(error) => {
                let _ = message_tx.send(GestureServiceMessage::DeviceError(error));
                std::thread::sleep(Duration::from_millis(120));
            }
        }
    }
}
