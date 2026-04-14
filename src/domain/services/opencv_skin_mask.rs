//! Сегментация области кожи через OpenCV (BGR → HSV, `inRange`, медианное сглаживание).
//!
//! Включение: в `Cargo.toml` уже есть фича `opencv`. Сборка: `cargo build --features opencv`.
//! Нужны установленные OpenCV и LLVM/Clang (см. [opencv-rust](https://github.com/twistedfall/opencv-rust)).
//! Если `build-script` падает с `STATUS_DLL_NOT_FOUND`, добавьте `clang` в `PATH` или поставьте LLVM.

use opencv::core::{in_range, Mat, Scalar, Vec3b, CV_8UC1, CV_8UC3};
use opencv::imgproc;
use opencv::prelude::*;

/// Бинарная маска кожи (`true` = кожа). Размер `width * height`.
pub fn skin_mask_opencv(rgb: &[u8], width: usize, height: usize) -> opencv::Result<Vec<bool>> {
    let expected = width * height * 3;
    if rgb.len() < expected {
        return Ok(vec![false; width * height]);
    }

    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 3;
            pixels.push(Vec3b([rgb[i + 2], rgb[i + 1], rgb[i]]));
        }
    }

    let bgr = Mat::new_rows_cols_with_data(
        height as i32,
        width as i32,
        pixels.as_slice(),
    )?;

    let mut hsv = Mat::default();
    imgproc::cvt_color(&bgr, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let mut mask = Mat::default();
    // H: 0–25 (OpenCV масштабирует H до ~0–180), S/V широкие под разное освещение.
    let lower = Scalar::new(0., 40., 60., 0.);
    let upper = Scalar::new(25., 255., 255., 0.);
    in_range(&hsv, &lower, &upper, &mut mask)?;

    let mut blurred = Mat::default();
    imgproc::median_blur(&mask, &mut blurred, 5)?;

    let mut out = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let v = *blurred.at_2d::<u8>(y as i32, x as i32)?;
            out[y * width + x] = v > 0;
        }
    }

    Ok(out)
}
