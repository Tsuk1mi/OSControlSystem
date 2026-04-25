//! Упрощённая оценка 2D-landmarks кисти по цвету кожи и радиальному профилю контура ладони.
//! Не заменяет MediaPipe, но даёт стабильный набор точек для геометрического классификатора.
//! Зона лица в кадре маскируется, кожный компонент выбирается по смещению вниз (см. `face_exclusion`).

use std::f64::consts::PI;

use crate::gesture_os_control::domain::services::face_exclusion;

/// 21 точка в стиле MediaPipe Hands (x, y в пикселях исходного кадра, z=0).
#[derive(Clone, Debug)]
pub struct HandLandmarks {
    pub points: [[f64; 3]; 21],
    pub palm_center: [f64; 3],
    pub wrist: [f64; 3],
}

/// Оценка landmarks; `None`, если рука не найдена или данные нестабильны.
pub fn estimate_hand_landmarks(rgb: &[u8], width: usize, height: usize) -> Option<HandLandmarks> {
    let (sw, sh) = (160_usize, 120_usize);
    let small = downsample_rgb(rgb, width, height, sw, sh);
    let mut mask = skin_mask_preferred(&small, sw, sh);
    face_exclusion::apply_to_skin_mask(&mut mask, sw, sh);
    let component = best_scored_hand_skin_component(&mask, sw, sh)?;
    let (centroid_x, centroid_y, area) = component_stats(&component, sw, sh);
    if area < 240 {
        return None;
    }

    let bbox = bounding_box(&component, sw, sh)?;
    let wrist_x = (bbox.0 + bbox.2 / 2) as f64;
    let wrist_y = (bbox.1 + bbox.3) as f64;

    let peaks = radial_distance_peaks(&component, centroid_x, centroid_y, sw, sh);
    if peaks.len() < 3 {
        return None;
    }

    let mut points = [[0.0_f64; 3]; 21];
    points[0] = [wrist_x, wrist_y, 0.0];
    points[9] = [centroid_x, centroid_y, 0.0];

    let tips = select_five_tips(&peaks, centroid_x, centroid_y, wrist_x, wrist_y);
    interpolate_finger_chain(
        &mut points,
        0,
        1,
        2,
        3,
        4,
        wrist_x,
        wrist_y,
        tips[0].0,
        tips[0].1,
    );
    interpolate_finger_chain(
        &mut points,
        0,
        5,
        6,
        7,
        8,
        centroid_x,
        centroid_y,
        tips[1].0,
        tips[1].1,
    );
    interpolate_finger_chain(
        &mut points,
        0,
        9,
        10,
        11,
        12,
        centroid_x,
        centroid_y,
        tips[2].0,
        tips[2].1,
    );
    interpolate_finger_chain(
        &mut points,
        0,
        13,
        14,
        15,
        16,
        centroid_x,
        centroid_y,
        tips[3].0,
        tips[3].1,
    );
    interpolate_finger_chain(
        &mut points,
        0,
        17,
        18,
        19,
        20,
        centroid_x,
        centroid_y,
        tips[4].0,
        tips[4].1,
    );

    let palm_center = [centroid_x, centroid_y, 0.0];
    let wrist = [wrist_x, wrist_y, 0.0];

    let sx = width as f64 / sw as f64;
    let sy = height as f64 / sh as f64;
    for p in &mut points {
        p[0] *= sx;
        p[1] *= sy;
    }

    let out = HandLandmarks {
        points,
        palm_center: [palm_center[0] * sx, palm_center[1] * sy, 0.0],
        wrist: [wrist[0] * sx, wrist[1] * sy, 0.0],
    };

    if !hand_landmarks_plausible(&out, width, height) {
        return None;
    }

    Some(out)
}

/// Отсекает «кисть» из случайного кожного пятна без реальной руки в кадре.
pub fn hand_landmarks_plausible(
    lm: &HandLandmarks,
    frame_width: usize,
    frame_height: usize,
) -> bool {
    let min_dim = frame_width.min(frame_height) as f64;
    if min_dim < 1.0 {
        return false;
    }

    let wrist = lm.wrist;
    let mid_mcp = lm.points[9];
    let scale = distance_xy(&wrist, &mid_mcp).max(1.0e-3);

    // Диапазон шире: MediaPipe и дальняя рука дают меньший масштаб, чем «идеальная» классика.
    let scale_min = min_dim * 0.030;
    let scale_max = min_dim * 0.60;
    if !(scale_min..=scale_max).contains(&scale) {
        return false;
    }

    let palm_c = lm.palm_center;
    let wrist_to_palm = distance_xy(&wrist, &palm_c);
    if wrist_to_palm < scale * 0.08 || wrist_to_palm > scale * 1.60 {
        return false;
    }

    true
}

fn distance_xy(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

fn downsample_rgb(rgb: &[u8], w: usize, h: usize, tw: usize, th: usize) -> Vec<u8> {
    let mut out = vec![0_u8; tw * th * 3];
    for ty in 0..th {
        for tx in 0..tw {
            let sx = (tx * w) / tw;
            let sy = (ty * h) / th;
            let si = (sy * w + sx) * 3;
            let di = (ty * tw + tx) * 3;
            if si + 2 < rgb.len() && di + 2 < out.len() {
                out[di..di + 3].copy_from_slice(&rgb[si..si + 3]);
            }
        }
    }
    out
}

fn skin_mask_preferred(rgb: &[u8], w: usize, h: usize) -> Vec<bool> {
    #[cfg(feature = "opencv")]
    {
        if let Ok(m) =
            crate::gesture_os_control::domain::services::opencv_skin_mask::skin_mask_opencv(
                rgb, w, h,
            )
        {
            let on = m.iter().copied().filter(|x| x).count();
            if on >= 120 {
                return m;
            }
        }
    }
    skin_mask_classic(rgb, w, h)
}

fn skin_mask_classic(rgb: &[u8], w: usize, h: usize) -> Vec<bool> {
    let mut mask = vec![false; w * h];
    for i in 0..w * h {
        let o = i * 3;
        if o + 2 >= rgb.len() {
            continue;
        }
        let r = rgb[o] as i32;
        let g = rgb[o + 1] as i32;
        let b = rgb[o + 2] as i32;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        if r > 95 && g > 40 && b > 20 && (r - g) > 15 && (r - b) > 15 && (max - min) > 15 {
            mask[i] = true;
        }
    }
    mask
}

/// Несколько кожных компонент: выбираем ту, что ближе к типичному положению кисти
/// (ниже по кадру, не «как лицо» — круглое пятно вверху).
fn best_scored_hand_skin_component(mask: &[bool], w: usize, h: usize) -> Option<Vec<bool>> {
    const MIN_AREA: usize = 160;
    let components = enumerate_skin_components(mask, w, h, MIN_AREA);
    if components.is_empty() {
        return None;
    }
    let h_f = h as f64;
    let mut best: Option<(f64, Vec<bool>)> = None;
    for cells in components {
        let score = score_component_hand_likeness(&cells, w, h, h_f);
        let comp = skin_cells_to_mask(&cells, mask.len());
        match &best {
            None => best = Some((score, comp)),
            Some((s, _)) if score > *s => best = Some((score, comp)),
            _ => {}
        }
    }
    best.map(|(_, m)| m)
}

fn enumerate_skin_components(
    mask: &[bool],
    w: usize,
    h: usize,
    min_area: usize,
) -> Vec<Vec<usize>> {
    let mut visited = vec![false; mask.len()];
    let mut out = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        let mut stack = vec![start];
        visited[start] = true;
        let mut cells = Vec::new();
        while let Some(idx) = stack.pop() {
            cells.push(idx);
            let x = idx % w;
            let y = idx / w;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if nx >= w || ny >= h {
                    continue;
                }
                let ni = ny * w + nx;
                if mask[ni] && !visited[ni] {
                    visited[ni] = true;
                    stack.push(ni);
                }
            }
        }
        if cells.len() >= min_area {
            out.push(cells);
        }
    }
    out
}

fn skin_cells_to_mask(cells: &[usize], len: usize) -> Vec<bool> {
    let mut c = vec![false; len];
    for &i in cells {
        c[i] = true;
    }
    c
}

fn score_component_hand_likeness(cells: &[usize], w: usize, h: usize, h_f: f64) -> f64 {
    let n = cells.len();
    if n < 160 {
        return 0.0;
    }
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0_usize;
    let mut max_y = 0_usize;
    let mut sy = 0.0_f64;
    for &idx in cells {
        let x = idx % w;
        let y = idx / w;
        sy += y as f64;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let cy = sy / n as f64;
    let bw = (max_x - min_x + 1) as f64;
    let bh = (max_y - min_y + 1) as f64;
    let long = bw.max(bh);
    let short = bw.min(bh);
    let aspect = if long > 0.0 { short / long } else { 0.0 };
    let vertical = (cy / h_f).powi(2);
    let top_round_penalty = if cy < 0.30 * h_f && aspect > 0.52 && n > 1_800 {
        0.45
    } else {
        1.0
    };
    let elong = (1.0 - aspect).clamp(0.0, 1.0);
    (n as f64).ln_1p() * (0.12 + 0.88 * vertical) * top_round_penalty * (1.0 + 0.18 * elong)
}

fn component_stats(component: &[bool], w: usize, _h: usize) -> (f64, f64, usize) {
    let mut n = 0_usize;
    let mut sx = 0_f64;
    let mut sy = 0_f64;
    for (i, on) in component.iter().enumerate() {
        if *on {
            n += 1;
            sx += (i % w) as f64;
            sy += (i / w) as f64;
        }
    }
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    (sx / n as f64, sy / n as f64, n)
}

fn bounding_box(component: &[bool], w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0_usize;
    let mut max_y = 0_usize;
    let mut any = false;
    for (i, on) in component.iter().enumerate() {
        if *on {
            any = true;
            let x = i % w;
            let y = i / w;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    if !any {
        return None;
    }
    Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
}

fn ray_boundary_distance(
    component: &[bool],
    cx: f64,
    cy: f64,
    w: usize,
    h: usize,
    angle: f64,
) -> f64 {
    let (vx, vy) = (angle.cos(), angle.sin());
    let mut t = 0.0_f64;
    let mut last = 0.0_f64;
    loop {
        let x = (cx + vx * t).round() as isize;
        let y = (cy + vy * t).round() as isize;
        if x < 0 || y < 0 || x >= w as isize || y >= h as isize {
            return last;
        }
        let xi = x as usize;
        let yi = y as usize;
        let idx = yi * w + xi;
        if component.get(idx).copied().unwrap_or(false) {
            last = t;
        }
        t += 1.0;
        if t > (w.max(h) as f64) * 1.5 {
            return last;
        }
    }
}

fn radial_distance_peaks(
    component: &[bool],
    cx: f64,
    cy: f64,
    w: usize,
    h: usize,
) -> Vec<(f64, f64)> {
    let steps = 56;
    let mut radii = vec![0.0_f64; steps];
    for i in 0..steps {
        let angle = (i as f64) * 2.0 * PI / steps as f64;
        radii[i] = ray_boundary_distance(component, cx, cy, w, h, angle);
    }

    let mut smooth = vec![0.0_f64; steps];
    for i in 0..steps {
        let mut acc = 0.0;
        for k in -3..=3 {
            let j = (i as isize + k).rem_euclid(steps as isize) as usize;
            acc += radii[j];
        }
        smooth[i] = acc / 7.0;
    }

    let median = {
        let mut sorted = smooth.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[steps / 2]
    };

    let mut peaks = Vec::new();
    for i in 0..steps {
        let prev = smooth[(i + steps - 1) % steps];
        let next = smooth[(i + 1) % steps];
        let cur = smooth[i];
        if cur > prev && cur > next && cur > median + 4.0 {
            let angle = (i as f64) * 2.0 * PI / steps as f64;
            let px = cx + angle.cos() * cur;
            let py = cy + angle.sin() * cur;
            peaks.push((px, py, cur, angle));
        }
    }

    peaks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    let mut filtered: Vec<(f64, f64, f64)> = Vec::new();
    for p in peaks {
        if filtered.iter().all(|q| {
            let da = (p.3 - q.2).abs().min(2.0 * PI - (p.3 - q.2).abs());
            da > 0.35
        }) {
            filtered.push((p.0, p.1, p.3));
            if filtered.len() >= 6 {
                break;
            }
        }
    }
    filtered.into_iter().map(|p| (p.0, p.1)).collect()
}

fn select_five_tips(peaks: &[(f64, f64)], cx: f64, cy: f64, wx: f64, wy: f64) -> [(f64, f64); 5] {
    let mut scored: Vec<((f64, f64), f64)> = peaks
        .iter()
        .copied()
        .map(|p| {
            let thumb_score = ((p.0 - wx).powi(2) + (p.1 - wy).powi(2)).sqrt();
            let spread = ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt();
            (p, thumb_score + spread * 0.15)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut tips = [(cx, cy); 5];
    for (i, item) in scored.iter().take(5).enumerate() {
        tips[i] = item.0;
    }
    tips
}

fn interpolate_finger_chain(
    points: &mut [[f64; 3]; 21],
    _wrist_idx: usize,
    mcp_idx: usize,
    pip_idx: usize,
    dip_idx: usize,
    tip_idx: usize,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
) {
    points[mcp_idx] = [ax + (bx - ax) * 0.35, ay + (by - ay) * 0.35, 0.0];
    points[pip_idx] = [ax + (bx - ax) * 0.62, ay + (by - ay) * 0.62, 0.0];
    points[dip_idx] = [ax + (bx - ax) * 0.82, ay + (by - ay) * 0.82, 0.0];
    points[tip_idx] = [bx, by, 0.0];
}
