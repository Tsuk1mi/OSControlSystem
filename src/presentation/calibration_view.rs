use eframe::egui;

use super::gesture_view_model::GestureViewModel;
use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::command::OsCommand;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

pub fn render_gesture_controls(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("cam_pick")
            .selected_text(view_model.selected_video_input())
            .width(200.0)
            .show_ui(ui, |ui| {
                for name in view_model.available_video_inputs().to_vec() {
                    if ui
                        .selectable_label(view_model.selected_video_input() == name, &name)
                        .clicked()
                    {
                        view_model.set_selected_video_input(&name);
                    }
                }
            });

        let mut mirror = view_model.gesture_mirror_horizontal();
        if ui.checkbox(&mut mirror, "зеркало").changed() {
            view_model.set_gesture_mirror_horizontal(mirror);
        }

        ui.label("чувств.");
        let mut sens = view_model.gesture_sensitivity();
        if ui
            .add(egui::Slider::new(&mut sens, 0.10..=1.00).step_by(0.01))
            .changed()
        {
            view_model.set_gesture_sensitivity(sens);
        }

        ui.label("пауза после жеста (с)");
        let mut cd = view_model.gesture_cooldown_secs();
        if ui
            .add(egui::Slider::new(&mut cd, 0.0..=8.0).step_by(0.1))
            .changed()
        {
            view_model.set_gesture_cooldown_secs(cd);
        }

        ui.separator();

        for (label, nw, nh) in [("640×480", 640, 480), ("1280×720", 1280, 720)] {
            if ui.button(label).clicked() {
                view_model.set_gesture_camera_resolution(nw, nh);
            }
        }

        ui.label("fps");
        let mut fps = view_model.gesture_camera_fps() as f32;
        if ui
            .add(egui::Slider::new(&mut fps, 5.0..=60.0).step_by(1.0))
            .changed()
        {
            view_model.set_gesture_camera_fps(fps.round() as u32);
        }

        ui.separator();

        for (label, mode) in [
            ("стол", AppRunMode::Desktop),
            ("медиа", AppRunMode::Media),
            ("браузер", AppRunMode::Browser),
        ] {
            let sel = view_model.gesture_run_mode() == mode;
            if ui.selectable_label(sel, label).clicked() {
                view_model.set_gesture_run_mode(mode);
            }
        }
    });

    if let Some(stats) = view_model.gesture_pipeline_stats() {
        ui.label(format!(
            "FPS {:.1} · кадров {} · устойч. {:.2} · увер. {:.2}",
            stats.fps_smoothed, stats.frames_captured, stats.last_stability, stats.last_confidence
        ));
    }

    ui.horizontal(|ui| {
        let can_start = !view_model.is_gesture_camera_running();
        if ui
            .add_enabled(can_start, egui::Button::new("Старт"))
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
    });

    ui.add_space(6.0);
    ui.collapsing("Привязки жест → действие", |ui| {
        ui.label(
            "Три контекста (стол / медиа / браузер). Файл gesture_bindings.json рядом с exe подхватывается при запуске.",
        );
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Жест").strong());
                    ui.add_space(80.0);
                    ui.label(egui::RichText::new("Стол").strong());
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("Медиа").strong());
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("Браузер").strong());
                });
                ui.separator();
                for &gesture in GestureId::BINDABLE {
                    let g_label = gesture.user_trigger_ru().unwrap_or("?");
                    ui.horizontal(|ui| {
                        ui.label(g_label);
                        for mode in [
                            AppRunMode::Desktop,
                            AppRunMode::Media,
                            AppRunMode::Browser,
                        ] {
                            let current = view_model.gesture_command_map().lookup(mode, gesture);
                            egui::ComboBox::from_id_salt(format!("bind_{mode:?}_{gesture:?}"))
                                .width(150.0)
                                .selected_text(current.label_ru())
                                .show_ui(ui, |ui| {
                                    for &cmd in OsCommand::ALL {
                                        let sel = current == cmd;
                                        if ui.selectable_label(sel, cmd.label_ru()).clicked() {
                                            view_model
                                                .gesture_command_map_mut()
                                                .set_binding(mode, gesture, cmd);
                                        }
                                    }
                                });
                        }
                    });
                }
            });
        if ui.button("Сохранить привязки в файл").clicked() {
            if let Err(e) = view_model.save_gesture_bindings_to_file() {
                view_model.push_log_line(format!("не сохранено: {e}"));
            }
        }
    });
}
