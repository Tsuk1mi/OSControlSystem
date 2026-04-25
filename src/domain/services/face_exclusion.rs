//! Исключение типичной зоны лица в селфи-кадре, чтобы кожа лица не забиралась как «кисть»
//! в Classic CV и не подсказывала ложные руки вокруг лица в MediaPipe.

const ELLIPSE_CX_FRAC: f64 = 0.5;
const ELLIPSE_CY_FRAC: f64 = 0.22;
const ELLIPSE_RX_FRAC: f64 = 0.36;
const ELLIPSE_RY_FRAC: f64 = 0.30;

#[inline]
fn in_face_ellipse(x: f64, y: f64, w: f64, h: f64) -> bool {
    let cx = ELLIPSE_CX_FRAC * w;
    let cy = ELLIPSE_CY_FRAC * h;
    let rx = ELLIPSE_RX_FRAC * w;
    let ry = ELLIPSE_RY_FRAC * h;
    if rx < 1.0 || ry < 1.0 {
        return false;
    }
    let dx = (x - cx) / rx;
    let dy = (y - cy) / ry;
    dx * dx + dy * dy <= 1.0
}

/// Обнуляет маску кожи в зоне лица (для `estimate_hand_landmarks`).
pub fn apply_to_skin_mask(mask: &mut [bool], w: usize, h: usize) {
    let wf = w as f64;
    let hf = h as f64;
    for y in 0..h {
        for x in 0..w {
            if in_face_ellipse(x as f64 + 0.5, y as f64 + 0.5, wf, hf) {
                mask[y * w + x] = false;
            }
        }
    }
}

/// Затемняет пиксели (чёрный) в зоне лица на RGB-кадре (вход в MediaPipe).
pub fn apply_to_rgb8_blackout(rgb: &mut [u8], w: usize, h: usize) {
    let wf = w as f64;
    let hf = h as f64;
    for y in 0..h {
        for x in 0..w {
            if in_face_ellipse(x as f64 + 0.5, y as f64 + 0.5, wf, hf) {
                let i = (y * w + x) * 3;
                if i + 2 < rgb.len() {
                    rgb[i] = 0;
                    rgb[i + 1] = 0;
                    rgb[i + 2] = 0;
                }
            }
        }
    }
}
