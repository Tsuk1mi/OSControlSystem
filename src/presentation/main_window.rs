use eframe::egui::{self, RichText, Vec2};

use super::calibration_view::render_gesture_controls;
use super::gesture_view_model::GestureViewModel;

pub fn render_app(ctx: &egui::Context, view_model: &mut GestureViewModel) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(view_model.title());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(view_model.gesture_camera_status()).weak());
            });
        });
    });

    egui::SidePanel::right("gesture_log")
        .resizable(true)
        .default_width(280.0)
        .min_width(200.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Журнал").strong());
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in view_model.event_log().iter() {
                        ui.monospace(line);
                    }
                });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
            if let Some(tex) = view_model.preview_texture() {
                let max_w = ui.available_width().min(920.0);
                let sz = tex.size_vec2();
                let scale = (max_w / sz.x).min(1.0);
                let display = sz * scale;
                ui.image((tex.id(), display));
            } else {
                ui.add_sized(
                    Vec2::new(ui.available_width(), 220.0),
                    egui::Label::new(RichText::new("Нет видео — запустите камеру.").weak()),
                );
            }

            ui.add_space(8.0);
            render_gesture_controls(ui, view_model);
        });
    });
}
