use std::time::Duration;

use eframe::egui;

use super::gesture_view_model::GestureViewModel;
use super::main_window::render_app;

pub struct GestureOsControlApp {
    view_model: GestureViewModel,
}

impl Default for GestureOsControlApp {
    fn default() -> Self {
        Self {
            view_model: GestureViewModel::new(),
        }
    }
}

impl eframe::App for GestureOsControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.view_model.tick();
        self.view_model.sync_preview_texture(ctx);
        render_app(ctx, &mut self.view_model);
        let ms = if self.view_model.is_gesture_camera_running() {
            16
        } else {
            48
        };
        ctx.request_repaint_after(Duration::from_millis(ms));
    }
}
