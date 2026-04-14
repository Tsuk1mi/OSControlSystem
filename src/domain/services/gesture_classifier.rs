use std::collections::VecDeque;
use std::time::Instant;

use crate::gesture_os_control::domain::entities::gesture::{GestureResult, GestureType};
use crate::gesture_os_control::domain::entities::landmark::HandLandmarks;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

const HISTORY_MS: u128 = 520;

#[derive(Clone, Debug)]
pub struct GestureClassifierConfig {
    pub sensitivity: f32,
}

impl Default for GestureClassifierConfig {
    fn default() -> Self {
        Self { sensitivity: 0.72 }
    }
}

/// Классификатор жестов по landmarks и короткой истории ладони.
pub struct GestureClassifier {
    config: GestureClassifierConfig,
    palm_history: VecDeque<(Instant, f64, f64)>,
}

impl GestureClassifier {
    pub fn new(config: GestureClassifierConfig) -> Self {
        Self {
            config,
            palm_history: VecDeque::new(),
        }
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.config.sensitivity = sensitivity;
    }

    pub fn clear_palm_history(&mut self) {
        self.palm_history.clear();
    }

    pub fn classify(
        &mut self,
        landmarks: &HandLandmarks,
        frame_size: (u32, u32),
        now: Instant,
    ) -> GestureResult {
        let (fw, fh) = (frame_size.0 as f64, frame_size.1 as f64);
        let norm = normalize_landmarks(landmarks);
        let finger = finger_features(&norm, self.config.sensitivity);

        let palm_px = landmarks.palm_center[0];
        let palm_py = landmarks.palm_center[1];
        self.palm_history.push_back((now, palm_px, palm_py));
        while self
            .palm_history
            .front()
            .map(|(t, _, _)| now.duration_since(*t).as_millis() > HISTORY_MS)
            .unwrap_or(false)
        {
            self.palm_history.pop_front();
        }

        let dyn_id = detect_swipe(&self.palm_history, self.config.sensitivity, fw, fh);
        let static_id = classify_static_geom(&norm, self.config.sensitivity);

        let (gesture, gesture_type, confidence) = if let Some(swipe) = dyn_id {
            (swipe, GestureType::Dynamic, 0.78_f32)
        } else if static_id != GestureId::None {
            let boosted = (finger.confidence * 0.5 + 0.45).clamp(0.0, 1.0);
            (static_id, GestureType::Static, boosted)
        } else {
            (GestureId::None, GestureType::None, 0.0_f32)
        };

        GestureResult {
            gesture,
            confidence,
            gesture_type,
            timestamp: now,
        }
    }
}

#[derive(Clone, Debug)]
struct NormalizedLandmarks {
    /// Точки после переноса запястья в начало координат и масштабирования.
    pub points: [[f64; 3]; 21],
    pub scale: f64,
}

#[derive(Clone, Debug)]
struct FingerFeatures {
    pub thumb_extended: bool,
    pub index_extended: bool,
    pub middle_extended: bool,
    pub ring_extended: bool,
    pub pinky_extended: bool,
    pub confidence: f32,
}

fn normalize_landmarks(lm: &HandLandmarks) -> NormalizedLandmarks {
    let wrist = lm.wrist;
    let middle_mcp = lm.points[9];
    let scale = distance3(&wrist, &middle_mcp).max(1.0e-3);
    let mut points = [[0.0_f64; 3]; 21];
    for (i, p) in lm.points.iter().enumerate() {
        points[i][0] = (p[0] - wrist[0]) / scale;
        points[i][1] = (p[1] - wrist[1]) / scale;
        points[i][2] = 0.0;
    }
    NormalizedLandmarks { points, scale }
}

fn finger_features(norm: &NormalizedLandmarks, sensitivity: f32) -> FingerFeatures {
    let s = sensitivity.clamp(0.05, 0.99) as f64;
    let ext = |mcp: usize, pip: usize, _dip: usize, tip: usize| -> (bool, f32) {
        let d_tip = distance3(&norm.points[0], &norm.points[tip]);
        let d_pip = distance3(&norm.points[0], &norm.points[pip]).max(1.0e-6);
        let ratio = d_tip / d_pip;
        let chain = distance3(&norm.points[mcp], &norm.points[tip]);
        let spread = (chain / norm.scale.max(1.0e-6)).min(2.5);
        let threshold = 1.05 - 0.26 * (1.0 - s);
        let spread_min = 0.24 + 0.10 * (1.0 - s);
        let extended = ratio > threshold && spread > spread_min;
        let conf = ((ratio - threshold) / 0.55).clamp(0.0, 1.0) as f32;
        (extended, conf * spread as f32)
    };

    let (thumb_extended, c0) = thumb_ext_special(norm, s);
    let (index_extended, c1) = ext(5, 6, 7, 8);
    let (middle_extended, c2) = ext(9, 10, 11, 12);
    let (ring_extended, c3) = ext(13, 14, 15, 16);
    let (pinky_extended, c4) = ext(17, 18, 19, 20);

    let mut confidence = (c0 + c1 + c2 + c3 + c4) / 5.0;
    if confidence.is_nan() {
        confidence = 0.0;
    }

    FingerFeatures {
        thumb_extended,
        index_extended,
        middle_extended,
        ring_extended,
        pinky_extended,
        confidence,
    }
}

/// Большой палец: цепочка 2→3→4, а не как у остальных пальцев (1→2→3→4).
fn thumb_ext_special(norm: &NormalizedLandmarks, s: f64) -> (bool, f32) {
    let tip = norm.points[4];
    let ip = norm.points[3];
    let mcp = norm.points[2];
    let wrist = norm.points[0];
    let d_tip_mcp = distance3(&tip, &mcp);
    let d_ip_mcp = distance3(&ip, &mcp).max(1.0e-6);
    let ratio = d_tip_mcp / d_ip_mcp;
    let threshold = 1.10 - 0.14 * (1.0 - s);
    let radial = distance3(&tip, &wrist) / distance3(&mcp, &wrist).max(1.0e-6);
    let extended = ratio > threshold && radial > 1.07;
    let conf = ((ratio - threshold) / 0.4).clamp(0.0, 1.0) as f32;
    (extended, conf * radial.min(2.0) as f32)
}

/// Отношение «кончик–запястье» к «PIP–запястье» (устойчивее эвристических landmarks).
fn tip_to_wrist_vs_pip(norm: &NormalizedLandmarks, pip: usize, tip: usize) -> f64 {
    let d_tip = distance3(&norm.points[tip], &norm.points[0]);
    let d_pip = distance3(&norm.points[pip], &norm.points[0]).max(1.0e-9);
    d_tip / d_pip
}

fn finger_curled_geom(norm: &NormalizedLandmarks, pip: usize, tip: usize, s: f64) -> bool {
    let r = tip_to_wrist_vs_pip(norm, pip, tip);
    r < 1.0 + 0.11 * (1.0 - s)
}

fn finger_straight_geom(norm: &NormalizedLandmarks, pip: usize, tip: usize, s: f64) -> bool {
    let r = tip_to_wrist_vs_pip(norm, pip, tip);
    r > 1.06 + 0.11 * (1.0 - s)
}

fn four_fingers_curled_count(norm: &NormalizedLandmarks, s: f64) -> usize {
    [
        finger_curled_geom(norm, 6, 8, s),
        finger_curled_geom(norm, 10, 12, s),
        finger_curled_geom(norm, 14, 16, s),
        finger_curled_geom(norm, 18, 20, s),
    ]
    .into_iter()
    .filter(|v| *v)
    .count()
}

fn four_fingers_straight_count(norm: &NormalizedLandmarks, s: f64) -> usize {
    [
        finger_straight_geom(norm, 6, 8, s),
        finger_straight_geom(norm, 10, 12, s),
        finger_straight_geom(norm, 14, 16, s),
        finger_straight_geom(norm, 18, 20, s),
    ]
    .into_iter()
    .filter(|v| *v)
    .count()
}

fn thumb_sticking_up_geom(norm: &NormalizedLandmarks, s: f64) -> bool {
    let tip = norm.points[4];
    let ip = norm.points[3];
    let mcp = norm.points[2];
    let dt = distance3(&tip, &norm.points[0]);
    let dm = distance3(&mcp, &norm.points[0]).max(1.0e-9);
    let radial = dt / dm;
    tip[1] < ip[1] - 0.05 && radial > 1.12 - 0.08 * (1.0 - s)
}

fn classify_static_geom(norm: &NormalizedLandmarks, sensitivity: f32) -> GestureId {
    let s = sensitivity.clamp(0.05, 0.99) as f64;

    let curled4 = four_fingers_curled_count(norm, s);
    let straight4 = four_fingers_straight_count(norm, s);
    let palm_span = distance3(&norm.points[5], &norm.points[17]).max(1.0e-3);
    let thumb_pinky = distance3(&norm.points[4], &norm.points[17]);

    let index_straight = finger_straight_geom(norm, 6, 8, s);
    let others_curled = finger_curled_geom(norm, 10, 12, s)
        && finger_curled_geom(norm, 14, 16, s)
        && finger_curled_geom(norm, 18, 20, s);

    // 1) Указание — один вытянутый указательный.
    if index_straight && others_curled {
        return GestureId::Pointing;
    }

    // 2) Открытая ладонь — три и более вытянутых или явно указ.+средний.
    if straight4 >= 3
        || (straight4 >= 2
            && finger_straight_geom(norm, 6, 8, s)
            && finger_straight_geom(norm, 10, 12, s))
    {
        return GestureId::OpenPalm;
    }

    // 3–4) Кулак vs большой палец: оба с согнутыми пальцами; различие — «большой вверх» и отступ от мизинца.
    if curled4 >= 3 {
        let up = thumb_sticking_up_geom(norm, s);
        if up && thumb_pinky > 0.38 * palm_span {
            return GestureId::ThumbUp;
        }
        if !up && thumb_pinky < 0.48 * palm_span {
            return GestureId::ClosedFist;
        }
    }

    GestureId::None
}

fn detect_swipe(
    history: &VecDeque<(Instant, f64, f64)>,
    sensitivity: f32,
    frame_w: f64,
    frame_h: f64,
) -> Option<GestureId> {
    if history.len() < 5 {
        return None;
    }
    let first = history.front()?;
    let last = history.back()?;
    let dx = (last.1 - first.1) / frame_w.max(1.0);
    let dy = (last.2 - first.2) / frame_h.max(1.0);
    let thr = 0.085 - 0.04 * sensitivity as f64;
    if dx.abs() < thr {
        return None;
    }
    if dy.abs() > 0.22 {
        return None;
    }
    if dx > 0.0 {
        Some(GestureId::SwipeRight)
    } else {
        Some(GestureId::SwipeLeft)
    }
}

fn distance3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

