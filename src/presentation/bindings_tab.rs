use eframe::egui::{self, RichText};

use crate::gesture_os_control::AppRunMode;
use crate::gesture_os_control::domain::entities::command::OsCommand;
use crate::gesture_os_control::domain::entities::context::ContextRule;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

use super::gesture_view_model::GestureViewModel;

pub fn render_bindings_tab(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    render_bindings_editor(ui, view_model);
    ui.add_space(12.0);
    render_context_rules_editor(ui, view_model);
}

fn render_bindings_editor(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Назначение действий на жесты").strong());
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Жест").strong());
                    ui.add_space(60.0);
                    ui.label(RichText::new("Стол").strong());
                    ui.add_space(40.0);
                    ui.label(RichText::new("Медиа").strong());
                    ui.add_space(40.0);
                    ui.label(RichText::new("Браузер").strong());
                });
                ui.separator();

                for &gesture in GestureId::BINDABLE {
                    let g_label = gesture.user_trigger_ru().unwrap_or("?");
                    ui.horizontal(|ui| {
                        ui.label(g_label);
                        for mode in [AppRunMode::Desktop, AppRunMode::Media, AppRunMode::Browser] {
                            let current = view_model.gesture_command_map().lookup(mode, gesture);
                            egui::ComboBox::from_id_salt(format!("bind_{mode:?}_{gesture:?}"))
                                .width(170.0)
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

        ui.add_space(8.0);
        if ui.button("Сохранить привязки").clicked() {
            if let Err(error) = view_model.save_gesture_bindings_to_file() {
                view_model.push_log_line(format!("не сохранено: {error}"));
            }
        }
    });
}

fn render_context_rules_editor(ui: &mut egui::Ui, view_model: &mut GestureViewModel) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Автоопределение контекста").strong());
            if ui.button("Добавить правило").clicked() {
                view_model.add_context_rule();
            }
            if ui.button("Сохранить правила").clicked() {
                if let Err(error) = view_model.save_context_rules_to_file() {
                    view_model.push_log_line(format!("не сохранено: {error}"));
                }
            }
        });
        ui.separator();
        ui.label(RichText::new("Матчинг: process contains + title contains -> mode").weak());

        let mut changed = false;
        let mut remove_idx = None;
        let total = view_model.context_rules().len();
        for index in 0..total {
            let mut remove_clicked = false;
            {
                let rules = view_model.context_rules_mut();
                let rule: &mut ContextRule = &mut rules[index];
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut rule.enabled, "").changed();
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut rule.name)
                                    .hint_text("Имя правила")
                                    .desired_width(160.0),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut rule.process_name_contains)
                                    .hint_text("process: chrome")
                                    .desired_width(140.0),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut rule.window_title_contains)
                                    .hint_text("title: youtube")
                                    .desired_width(180.0),
                            )
                            .changed();

                        egui::ComboBox::from_id_salt(format!("rule_mode_{index}"))
                            .selected_text(rule.mode.label_ru())
                            .show_ui(ui, |ui| {
                                for mode in
                                    [AppRunMode::Desktop, AppRunMode::Media, AppRunMode::Browser]
                                {
                                    if ui
                                        .selectable_label(rule.mode == mode, mode.label_ru())
                                        .clicked()
                                    {
                                        rule.mode = mode;
                                        changed = true;
                                    }
                                }
                            });

                        remove_clicked = ui.button("Удалить").clicked();
                    });
                });
            }
            if remove_clicked {
                remove_idx = Some(index);
            }
        }

        if let Some(index) = remove_idx {
            view_model.remove_context_rule(index);
        } else if changed {
            view_model.commit_context_rules("");
        }
    });
}
