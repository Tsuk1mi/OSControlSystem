use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, StrokeKind, Vec2};

use super::gesture_view_model::GestureViewModel;

pub fn render_video_tab(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    render_video_controls(ui, view_model);
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].vertical(|ui| {
            ui.label(RichText::new("Видеопоток и overlay").strong());
            ui.label(
                RichText::new(
                    "Overlay показывает bbox, landmarks, confidence, motion и resolved context в реальном времени.",
                )
                .small()
                .weak(),
            );
            ui.add_space(6.0);
            render_preview(ui, view_model);
        });

        columns[1].vertical(|ui| {
            ui.label(RichText::new("Диагностика").strong());
            ui.add_space(6.0);
            render_debug_summary(ui, view_model);
            ui.add_space(8.0);
            ui.label(RichText::new("Журнал").strong());
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(460.0)
                .show(ui, |ui| {
                    for line in view_model.event_log().iter() {
                        ui.monospace(line);
                    }
                });
        });
    });
}

fn render_video_controls(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    ui.horizontal(|ui| {
        let can_start = !view_model.is_gesture_camera_running();
        if ui
            .add_enabled(can_start, egui::Button::new("Старт камеры"))
            .clicked()
        {
            view_model.start_gesture_camera();
        }
        if ui
            .add_enabled(!can_start, egui::Button::new("Стоп"))
            .clicked()
        {
            view_model.stop_gesture_camera();
        }
        ui.label(RichText::new(view_model.gesture_camera_status()).weak());
    });
}

fn render_preview(ui: &mut egui::Ui, view_model: &GestureViewModel) {
    let available_width = ui.available_width().min(920.0);
    if let Some(tex) = view_model.preview_texture() {
        let tex_size = tex.size_vec2();
        let scale = (available_width / tex_size.x).min(1.0);
        let display_size = tex_size * scale;
        let image = egui::Image::new((tex.id(), display_size)).sense(Sense::hover());
        let response = ui.add(image);
        let rect = response.rect;
        let painter = ui.painter_at(rect);

        if let Some(debug) = view_model.latest_debug_frame() {
            if let Some([x, y, w, h]) = debug.bounding_box {
                let bbox = Rect::from_min_size(
                    map_point(rect, debug.frame_width, debug.frame_height, x, y),
                    Vec2::new(
                        w / debug.frame_width.max(1) as f32 * rect.width(),
                        h / debug.frame_height.max(1) as f32 * rect.height(),
                    ),
                );
                painter.rect_stroke(
                    bbox,
                    6.0,
                    egui::Stroke::new(2.0, Color32::from_rgb(80, 200, 120)),
                    StrokeKind::Outside,
                );
            }

            for point in &debug.landmarks {
                let center = map_point(
                    rect,
                    debug.frame_width,
                    debug.frame_height,
                    point[0],
                    point[1],
                );
                painter.circle_filled(center, 3.2, Color32::from_rgb(255, 170, 80));
            }

            let overlay = format!(
                "{} · {:.0}% · {}",
                debug.raw_gesture.user_trigger_ru().unwrap_or("нет жеста"),
                debug.raw_confidence * 100.0,
                debug.context_summary
            );
            painter.text(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::Align2::LEFT_TOP,
                overlay,
                egui::FontId::proportional(15.0),
                Color32::WHITE,
            );
        }
    } else {
        ui.add_sized(
            Vec2::new(ui.available_width(), 260.0),
            egui::Label::new(RichText::new("Нет видео — запустите камеру.").weak()),
        );
    }
}

fn render_debug_summary(ui: &mut egui::Ui, view_model: &GestureViewModel) {
    egui::Grid::new("video_debug_grid")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Статус");
            ui.monospace(view_model.gesture_camera_status());
            ui.end_row();

            ui.label("Backend");
            let backend_label = view_model
                .latest_debug_frame()
                .and_then(|frame| frame.backend_label.clone())
                .unwrap_or_else(|| view_model.gesture_backend_kind().label_ru().to_owned());
            ui.label(backend_label);
            ui.end_row();

            if let Some(debug) = view_model.latest_debug_frame() {
                ui.label("Фильтр");
                ui.label(debug.filter_status.label_ru());
                ui.end_row();

                ui.label("Причина");
                ui.label(&debug.filter_reason);
                ui.end_row();

                ui.label("Контекст");
                ui.label(&debug.context_summary);
                ui.end_row();

                ui.label("Жест");
                ui.label(debug.raw_gesture.user_trigger_ru().unwrap_or("нет"));
                ui.end_row();

                ui.label("Уверенность");
                ui.label(format!("{:.0}%", debug.raw_confidence * 100.0));
                ui.end_row();

                ui.label("Motion");
                let motion = debug
                    .detected_motion
                    .map(|motion| format!("dx {:+.3}, dy {:+.3}", motion[0], motion[1]))
                    .unwrap_or_else(|| "н/д".to_owned());
                ui.label(motion);
                ui.end_row();

                ui.label("Landmarks");
                ui.label(debug.landmarks.len().to_string());
                ui.end_row();
            }

            if let Some(stats) = view_model.gesture_pipeline_stats() {
                ui.label("FPS");
                ui.label(format!("{:.1}", stats.fps_smoothed));
                ui.end_row();

                ui.label("Кадров");
                ui.label(stats.frames_captured.to_string());
                ui.end_row();

                ui.label("Стабильность");
                ui.label(format!("{:.0}%", stats.last_stability * 100.0));
                ui.end_row();
            }
        });
}

fn map_point(rect: Rect, frame_width: u32, frame_height: u32, x: f32, y: f32) -> Pos2 {
    let fx = frame_width.max(1) as f32;
    let fy = frame_height.max(1) as f32;
    Pos2::new(
        rect.left() + x / fx * rect.width(),
        rect.top() + y / fy * rect.height(),
    )
}
