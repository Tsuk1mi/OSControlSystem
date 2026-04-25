use eframe::egui::{self, RichText};

use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::context::ContextDetectionMode;
use crate::gesture_os_control::domain::entities::gesture_backend::GestureBackendKind;

use super::gesture_view_model::GestureViewModel;

pub fn render_settings_tab(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
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
        ui.label(RichText::new(view_model.gesture_camera_status()).weak());
    });

    ui.add_space(10.0);

    ui.columns(2, |columns| {
        columns[0].vertical(|ui| {
            render_camera_settings(ui, view_model);
            ui.add_space(10.0);
            render_recognition_settings(ui, view_model);
        });

        columns[1].vertical(|ui| {
            render_backend_settings(ui, view_model);
            ui.add_space(10.0);
            render_context_settings(ui, view_model);
        });
    });
}

fn render_camera_settings(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Камера").strong());
        ui.separator();

        egui::ComboBox::from_id_salt("cam_pick")
            .selected_text(view_model.selected_video_input())
            .width(220.0)
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
        if ui.checkbox(&mut mirror, "Зеркалировать").changed() {
            view_model.set_gesture_mirror_horizontal(mirror);
        }

        ui.horizontal(|ui| {
            ui.label("Разрешение");
            for (label, nw, nh) in [("640×480", 640, 480), ("1280×720", 1280, 720)] {
                if ui.button(label).clicked() {
                    view_model.set_gesture_camera_resolution(nw, nh);
                }
            }
        });

        let mut fps = view_model.gesture_camera_fps() as f32;
        if ui
            .add(
                egui::Slider::new(&mut fps, 5.0..=60.0)
                    .text("FPS")
                    .step_by(1.0),
            )
            .changed()
        {
            view_model.set_gesture_camera_fps(fps.round() as u32);
        }
    });
}

fn render_recognition_settings(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Распознавание").strong());
        ui.separator();
        ui.label(
            RichText::new(
                "Чувствительность влияет на статические жесты. Слишком высокое значение усиливает шум, слишком низкое — пропускает позы.",
            )
            .small()
            .weak(),
        );

        let mut sens = view_model.gesture_sensitivity();
        if ui
            .add(
                egui::Slider::new(&mut sens, 0.10..=1.00)
                    .text("Чувствительность")
                    .step_by(0.01),
            )
            .changed()
        {
            view_model.set_gesture_sensitivity(sens);
        }

        ui.label(
            RichText::new(
                "Пауза после жеста защищает от повторного срабатывания одного и того же движения.",
            )
            .small()
            .weak(),
        );
        let mut cd = view_model.gesture_cooldown_secs();
        if ui
            .add(
                egui::Slider::new(&mut cd, 0.0..=8.0)
                    .text("Пауза после жеста")
                    .step_by(0.1),
            )
            .changed()
        {
            view_model.set_gesture_cooldown_secs(cd);
        }
    });
}

fn render_backend_settings(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Gesture backend").strong());
        ui.separator();
        ui.label(
            RichText::new(
                "MediaPipe: Python 3.10–3.12 (на 3.13 колёс часто нет). pip: py -3.11 -m pip install -r python/mediapipe-requirements.txt. Или OSCONTROL_PYTHON=путь\\python.exe с mediapipe; скрипт: scripts\\install-mediapipe.ps1. Без пакета — Classic CV.",
            )
            .small()
            .weak(),
        );

        for kind in [GestureBackendKind::MediaPipe, GestureBackendKind::Classic] {
            let selected = view_model.gesture_backend_kind() == kind;
            if ui.selectable_label(selected, kind.label_ru()).clicked() {
                view_model.set_gesture_backend_kind(kind);
            }
            ui.label(RichText::new(kind.description_ru()).small().weak());
            ui.add_space(4.0);
        }

        let mut path = view_model.mediapipe_model_path().to_owned();
        if ui
            .add(
                egui::TextEdit::singleline(&mut path)
                    .hint_text("Резерв: путь к .task / model file")
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            view_model.set_mediapipe_model_path(path);
        }

        if let Some(debug) = view_model.latest_debug_frame() {
            if let Some(label) = &debug.backend_label {
                ui.label(RichText::new(label).small().weak());
            }
        }
    });
}

fn render_context_settings(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Контекст").strong());
        ui.separator();

        ui.horizontal(|ui| {
            for mode in [ContextDetectionMode::Manual, ContextDetectionMode::Auto] {
                let selected = view_model.context_detection_mode() == mode;
                if ui.selectable_label(selected, mode.label_ru()).clicked() {
                    view_model.set_context_detection_mode(mode);
                }
            }
        });

        ui.add_space(6.0);
        ui.label("Ручной режим");
        ui.horizontal(|ui| {
            for mode in [AppRunMode::Desktop, AppRunMode::Media, AppRunMode::Browser] {
                let selected = view_model.gesture_run_mode() == mode;
                if ui.selectable_label(selected, mode.label_ru()).clicked() {
                    view_model.set_gesture_run_mode(mode);
                }
            }
        });

        if let Some(debug) = view_model.latest_debug_frame() {
            ui.add_space(8.0);
            ui.label(format!("Сейчас: {}", debug.context_summary));
        } else {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Текущий resolved context появится после старта камеры.").weak(),
            );
        }
    });
}
