use eframe::egui::{self, RichText};

use super::bindings_tab::render_bindings_tab;
use super::gesture_view_model::{GestureTab, GestureViewModel};
use super::settings_tab::render_settings_tab;
use super::video_tab::render_video_tab;

pub fn render_app(ctx: &egui::Context, view_model: &mut GestureViewModel) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(view_model.title());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(view_model.gesture_camera_status()).weak());
                });
            });

            ui.horizontal(|ui| {
                for tab in [
                    GestureTab::Video,
                    GestureTab::Settings,
                    GestureTab::Bindings,
                ] {
                    let selected = view_model.current_tab() == tab;
                    if ui.selectable_label(selected, tab.label_ru()).clicked() {
                        view_model.set_current_tab(tab);
                    }
                }
            });
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| match view_model.current_tab() {
        GestureTab::Video => render_video_tab(ui, view_model),
        GestureTab::Settings => render_settings_tab(ui, view_model),
        GestureTab::Bindings => render_bindings_tab(ui, view_model),
    });
}
