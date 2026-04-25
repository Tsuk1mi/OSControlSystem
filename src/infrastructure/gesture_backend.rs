//! Два источника 21 точки кисти: **Classic** (эвристики по коже в `landmark`) и **MediaPipe**
//! (дочерний Python-процесс, см. `mediapipe_hands_helper.py`: legacy `solutions` или `tasks.HandLandmarker`).
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::gesture_os_control::application::dto::frame_dto::FrameDto;
use crate::gesture_os_control::application::dto::gesture_debug_dto::GestureDebugFrameDto;
use crate::gesture_os_control::application::ports::input::gesture_backend_port::{
    GestureBackendOutput, GestureBackendPort,
};
use crate::gesture_os_control::domain::entities::gesture_backend::GestureBackendKind;
use crate::gesture_os_control::domain::entities::landmark::{
    HandLandmarks, estimate_hand_landmarks, hand_landmarks_plausible,
};
use crate::gesture_os_control::domain::services::face_exclusion;

#[derive(Clone, Debug, Default)]
pub struct GestureBackendConfig {
    pub kind: GestureBackendKind,
    pub mediapipe_model_path: String,
}

const MEDIAPIPE_HELPER_FILE_NAME: &str = "oscontrolassistant_mediapipe_hands_helper.py";
const MEDIAPIPE_HELPER_SOURCE: &str = include_str!("mediapipe_hands_helper.py");

pub fn create_backend(config: &GestureBackendConfig) -> Box<dyn GestureBackendPort> {
    match config.kind {
        GestureBackendKind::Classic => Box::new(ClassicGestureBackend),
        GestureBackendKind::MediaPipe => Box::new(MediaPipeGestureBackend::new(
            config.mediapipe_model_path.clone(),
        )),
    }
}

struct ClassicGestureBackend;

impl GestureBackendPort for ClassicGestureBackend {
    fn backend_name(&self) -> &'static str {
        "Classic CV"
    }

    fn process_frame(&mut self, frame: &FrameDto) -> Result<GestureBackendOutput, String> {
        let landmarks =
            estimate_hand_landmarks(&frame.rgb8, frame.width as usize, frame.height as usize)
                .filter(|lm| {
                    hand_landmarks_plausible(lm, frame.width as usize, frame.height as usize)
                });
        Ok(GestureBackendOutput {
            debug_frame: debug_from_landmarks(frame, self.backend_name(), None, landmarks.as_ref()),
            landmarks,
            direct_gesture: None,
        })
    }
}

struct MediaPipeGestureBackend {
    model_path: String,
    helper: Option<MediaPipePythonHelper>,
    helper_label: Option<String>,
    availability_error: Option<String>,
    fallback: ClassicGestureBackend,
}

impl MediaPipeGestureBackend {
    fn new(model_path: String) -> Self {
        let (helper, helper_label, availability_error) = match MediaPipePythonHelper::spawn() {
            Ok(helper) => {
                let label = helper.label.clone();
                (Some(helper), Some(label), None)
            }
            Err(error) => (None, None, Some(error)),
        };
        Self {
            model_path,
            helper,
            helper_label,
            availability_error,
            fallback: ClassicGestureBackend,
        }
    }
}

impl GestureBackendPort for MediaPipeGestureBackend {
    fn backend_name(&self) -> &'static str {
        "MediaPipe"
    }

    fn process_frame(&mut self, frame: &FrameDto) -> Result<GestureBackendOutput, String> {
        if let Some(helper) = &mut self.helper {
            match helper.process_frame(frame) {
                Ok(Some(landmarks)) => {
                    let mut debug = debug_from_landmarks(
                        frame,
                        self.backend_name(),
                        Some(self.compose_backend_label(true)),
                        Some(&landmarks),
                    );
                    if !self.model_path.trim().is_empty() {
                        debug.backend_label = Some(format!(
                            "{} · model hint: {}",
                            debug.backend_label.unwrap_or_default(),
                            self.model_path
                        ));
                    }
                    return Ok(GestureBackendOutput {
                        debug_frame: debug,
                        landmarks: Some(landmarks),
                        direct_gesture: None,
                    });
                }
                Ok(None) => {
                    return Ok(GestureBackendOutput {
                        debug_frame: debug_from_landmarks(
                            frame,
                            self.backend_name(),
                            Some(self.compose_backend_label(true)),
                            None,
                        ),
                        landmarks: None,
                        direct_gesture: None,
                    });
                }
                Err(error) => {
                    self.availability_error = Some(error);
                    self.helper = None;
                }
            }
        }

        let mut output = self.fallback.process_frame(frame)?;
        output.debug_frame.backend_name = self.backend_name().to_owned();
        output.debug_frame.backend_label = Some(self.compose_backend_label(false));
        Ok(output)
    }
}

impl MediaPipeGestureBackend {
    fn compose_backend_label(&self, helper_ready: bool) -> String {
        let mut parts = Vec::new();
        if helper_ready {
            parts.push(
                self.helper_label
                    .clone()
                    .unwrap_or_else(|| "MediaPipe Hands".to_owned()),
            );
        } else {
            parts.push("fallback: Classic CV".to_owned());
            if let Some(error) = &self.availability_error {
                parts.push(error.clone());
            }
        }
        if !self.model_path.trim().is_empty() {
            parts.push(format!("path: {}", self.model_path));
        }
        parts.join(" · ")
    }
}

fn debug_from_landmarks(
    frame: &FrameDto,
    backend_name: &str,
    backend_label: Option<String>,
    landmarks: Option<&HandLandmarks>,
) -> GestureDebugFrameDto {
    let mut debug = GestureDebugFrameDto {
        frame_width: frame.width,
        frame_height: frame.height,
        backend_name: backend_name.to_owned(),
        backend_label,
        ..GestureDebugFrameDto::default()
    };

    if let Some(landmarks) = landmarks {
        debug.landmarks = landmarks
            .points
            .iter()
            .map(|point| [point[0] as f32, point[1] as f32])
            .collect();
        debug.bounding_box = landmarks_bbox(landmarks);
    }

    debug
}

fn landmarks_bbox(landmarks: &HandLandmarks) -> Option<[f32; 4]> {
    let first = landmarks.points.first()?;
    let mut min_x = first[0];
    let mut min_y = first[1];
    let mut max_x = first[0];
    let mut max_y = first[1];
    for point in landmarks.points.iter().skip(1) {
        min_x = min_x.min(point[0]);
        min_y = min_y.min(point[1]);
        max_x = max_x.max(point[0]);
        max_y = max_y.max(point[1]);
    }
    Some([
        min_x as f32,
        min_y as f32,
        (max_x - min_x).max(1.0) as f32,
        (max_y - min_y).max(1.0) as f32,
    ])
}

#[derive(Deserialize)]
struct HelperHello {
    ready: bool,
    backend: Option<String>,
    version: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct HelperResponse {
    ok: bool,
    landmarks: Option<Vec<[f32; 3]>>,
    handedness: Option<String>,
    error: Option<String>,
}

struct MediaPipePythonHelper {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    label: String,
}

/// `py -3.11 -u script` / `python.exe -u script` — без `-u` первая строка JSON на Windows
/// может «застрять» в буфере и чтение поймёт пустой stdout.
fn build_python_command(program: &str, extra_args: &[String], script: &Path) -> Command {
    let mut c = Command::new(program);
    if program.eq_ignore_ascii_case("py") {
        c.args(extra_args);
        c.arg("-u");
    } else {
        c.arg("-u");
        if !extra_args.is_empty() {
            c.args(extra_args);
        }
    }
    c.arg(script);
    c
}

fn join_mediapipe_attempt_errors(attempts: &[String]) -> String {
    if attempts.is_empty() {
        return "Не удалось запустить MediaPipe helper.".to_owned();
    }
    let s = attempts.join(" || ");
    if s.len() > 1_200 {
        format!("{}…", s.chars().take(1_200).collect::<String>())
    } else {
        s
    }
}

impl MediaPipePythonHelper {
    fn spawn() -> Result<Self, String> {
        let script_path = write_helper_script()?;
        let mut attempt_errors: Vec<String> = Vec::new();
        for (program, extra_args) in collect_python_launchers() {
            let mut command = build_python_command(&program, &extra_args, script_path.as_path());
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                // Piped stderr без чтения на Windows часто приводит к зависанию helper при объёмном выводе.
                .stderr(Stdio::inherit())
                .env("PYTHONUNBUFFERED", "1");
            #[cfg(windows)]
            {
                command.env("PYTHONUTF8", "1");
            }
            let cmd_label = if extra_args.is_empty() {
                program.clone()
            } else {
                format!("{program} {}", extra_args.join(" "))
            };

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(error) => {
                    attempt_errors.push(format!("{cmd_label}: {error}"));
                    continue;
                }
            };
            let Some(stdin) = child.stdin.take() else {
                let _ = child.kill();
                attempt_errors.push(format!("{cmd_label}: нет stdin у helper."));
                continue;
            };
            let Some(stdout) = child.stdout.take() else {
                let _ = child.kill();
                attempt_errors.push(format!("{cmd_label}: нет stdout у helper."));
                continue;
            };
            let mut stdout = BufReader::new(stdout);
            let hello: HelperHello = match read_json_line(&mut stdout) {
                Ok(h) => h,
                Err(e) => {
                    let status_note = child
                        .try_wait()
                        .ok()
                        .flatten()
                        .map(|s| format!(" (код выхода {s})"))
                        .unwrap_or_default();
                    let _ = child.kill();
                    let _ = child.wait();
                    attempt_errors.push(format!("{cmd_label}: {e}{status_note}"));
                    continue;
                }
            };
            if hello.ready {
                let label = match (hello.backend, hello.version) {
                    (Some(backend), Some(version)) => format!("{backend} {version}"),
                    (Some(backend), None) => backend,
                    _ => "MediaPipe Hands (python)".to_owned(),
                };
                return Ok(Self {
                    child,
                    stdin,
                    stdout,
                    label,
                });
            }
            let _ = child.kill();
            let _ = child.wait();
            let error = hello
                .error
                .unwrap_or_else(|| "MediaPipe helper не смог инициализироваться.".to_owned());
            attempt_errors.push(format!("{cmd_label}: {error}"));
        }
        Err(join_mediapipe_attempt_errors(&attempt_errors))
    }

    fn process_frame(&mut self, frame: &FrameDto) -> Result<Option<HandLandmarks>, String> {
        let (send_w, send_h, buf) = mediapipe_downscaled_rgb(frame);
        self.stdin
            .write_all(&send_w.to_le_bytes())
            .and_then(|_| self.stdin.write_all(&send_h.to_le_bytes()))
            .and_then(|_| self.stdin.write_all(&(buf.len() as u32).to_le_bytes()))
            .and_then(|_| self.stdin.write_all(&buf))
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("stdin MediaPipe helper: {error}"))?;

        let response: HelperResponse = read_json_line(&mut self.stdout)?;
        if !response.ok {
            return Err(response
                .error
                .unwrap_or_else(|| "MediaPipe helper вернул ошибку.".to_owned()));
        }

        let Some(landmarks) = response.landmarks else {
            return Ok(None);
        };
        if landmarks.len() < 21 {
            return Ok(None);
        }
        let mut hand = landmarks_from_normalized(frame, &landmarks);
        if let Some(label) = response.handedness {
            if let Some(hand_ref) = hand.as_mut() {
                if label.eq_ignore_ascii_case("left") {
                    hand_ref.palm_center[2] = -1.0;
                } else if label.eq_ignore_ascii_case("right") {
                    hand_ref.palm_center[2] = 1.0;
                }
            }
        }
        Ok(hand)
    }
}

impl Drop for MediaPipePythonHelper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn landmarks_from_normalized(frame: &FrameDto, landmarks: &[[f32; 3]]) -> Option<HandLandmarks> {
    if landmarks.len() < 21 {
        return None;
    }
    let fw = frame.width as f64;
    let fh = frame.height as f64;
    let z_scale = fw.min(fh);
    let mut points = [[0.0_f64; 3]; 21];
    for (index, point) in landmarks.iter().take(21).enumerate() {
        points[index] = [
            point[0] as f64 * fw,
            point[1] as f64 * fh,
            point[2] as f64 * z_scale,
        ];
    }
    let wrist = points[0];
    let palm_center = average_points(&points, &[0, 5, 9, 13, 17]);
    let hand = HandLandmarks {
        points,
        palm_center,
        wrist,
    };
    hand_landmarks_plausible(&hand, frame.width as usize, frame.height as usize).then_some(hand)
}

fn average_points(points: &[[f64; 3]; 21], indices: &[usize]) -> [f64; 3] {
    let mut sum = [0.0_f64; 3];
    for &index in indices {
        let point = points[index];
        sum[0] += point[0];
        sum[1] += point[1];
        sum[2] += point[2];
    }
    let n = indices.len().max(1) as f64;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Сначала каталог exe (лучше для антивирусов), затем temp.
fn write_helper_script() -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(mut p) = std::env::current_exe() {
        p.pop();
        p.push(MEDIAPIPE_HELPER_FILE_NAME);
        candidates.push(p);
    }
    let mut temp = std::env::temp_dir();
    temp.push(MEDIAPIPE_HELPER_FILE_NAME);
    candidates.push(temp);

    let mut last_err: Option<String> = None;
    for path in candidates {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::write(&path, MEDIAPIPE_HELPER_SOURCE) {
            Ok(()) => return Ok(path),
            Err(error) => last_err = Some(format!("`{}`: {error}", path.display())),
        }
    }
    Err(last_err.unwrap_or_else(|| "Не удалось записать MediaPipe helper.".to_owned()))
}

/// `OSCONTROL_PYTHON` = полный путь к `python.exe` с mediapipe — проверяется первым.
/// Далее `py` с версиями, где на Windows чаще есть колёса mediapipe (3.11 до общего `-3`).
#[cfg(windows)]
const PYTHON_WINDOWS: &[(&str, &[&str])] = &[
    ("py", &["-3.11"]),
    ("py", &["-3.10"]),
    ("py", &["-3.12"]),
    ("py", &["-3.9"]),
    ("py", &["-3"]),
    ("py", &["-3.13"]),
    ("python", &[]),
    // `python3` в PATH на Windows нередко — заглушка Microsoft Store (пустой stdout).
];

fn collect_python_launchers() -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    if let Ok(s) = std::env::var("OSCONTROL_PYTHON") {
        let t = s.trim();
        if !t.is_empty() {
            out.push((t.to_owned(), Vec::new()));
        }
    }
    #[cfg(windows)]
    {
        for &(prog, args) in PYTHON_WINDOWS {
            out.push((
                prog.to_owned(),
                args.iter().map(|s| (*s).to_owned()).collect(),
            ));
        }
    }
    #[cfg(not(windows))]
    {
        out.push(("python3".to_owned(), Vec::new()));
        out.push(("python".to_owned(), Vec::new()));
    }
    out
}

/// Узкая сторона кадра для IPC с Python: баланс скорости и детализации landmarks.
const MEDIAPIPE_HELPER_MAX_SIDE: u32 = 480;

/// По умолчанию лицо на кадре для MediaPipe не маскируем (иначе кисть у лица хуже).
/// Включить: `set OSCONTROL_FACE_MASK_MEDIAPIPE=1`
fn mediapipe_optional_face_mask_rgb(rgb: &mut [u8], w: usize, h: usize) {
    let on = std::env::var("OSCONTROL_FACE_MASK_MEDIAPIPE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if on {
        face_exclusion::apply_to_rgb8_blackout(rgb, w, h);
    }
}

fn mediapipe_downscaled_rgb(frame: &FrameDto) -> (u32, u32, Vec<u8>) {
    let w = frame.width.max(1);
    let h = frame.height.max(1);
    let m = w.max(h);
    if m <= MEDIAPIPE_HELPER_MAX_SIDE {
        let mut buf = frame.rgb8.clone();
        mediapipe_optional_face_mask_rgb(&mut buf, w as usize, h as usize);
        return (w, h, buf);
    }
    let scale = MEDIAPIPE_HELPER_MAX_SIDE as f64 / m as f64;
    let nw = ((w as f64) * scale).round().max(1.0) as u32;
    let nh = ((h as f64) * scale).round().max(1.0) as u32;
    let mut out = vec![0u8; (nw * nh * 3) as usize];
    for y in 0..nh {
        for x in 0..nw {
            let sx = (((x as f64 + 0.5) * w as f64 / nw as f64).floor() as u32).min(w - 1);
            let sy = (((y as f64 + 0.5) * h as f64 / nh as f64).floor() as u32).min(h - 1);
            let si = ((sy * w + sx) * 3) as usize;
            let di = ((y * nw + x) * 3) as usize;
            if si + 3 <= frame.rgb8.len() && di + 3 <= out.len() {
                out[di..di + 3].copy_from_slice(&frame.rgb8[si..si + 3]);
            }
        }
    }
    mediapipe_optional_face_mask_rgb(&mut out, nw as usize, nh as usize);
    (nw, nh, out)
}

fn read_json_line<T: DeserializeOwned>(reader: &mut BufReader<ChildStdout>) -> Result<T, String> {
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|error| format!("stdout MediaPipe helper: {error}"))?;
    if read == 0 || line.trim().is_empty() {
        return Err(
            "пустой stdout: процесс сразу завершился или не вывел JSON. \
             На Windows команда `python3` в PATH часто ведёт в Microsoft Store без реального Python — \
             поставьте python.org, выполните py -3.11 -m pip install mediapipe numpy, \
             либо задайте OSCONTROL_PYTHON=полный_путь\\python.exe"
                .to_owned(),
        );
    }
    let trimmed = line.trim();
    serde_json::from_str(trimmed).map_err(|error| {
        let preview: String = trimmed.chars().take(160).collect();
        format!("JSON MediaPipe helper: {error} (строка: {preview})")
    })
}
