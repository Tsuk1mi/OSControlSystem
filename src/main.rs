//! Точка входа: нативное окно Windows без консоли (`windows_subsystem`), запуск eframe/egui.
#![windows_subsystem = "windows"]

mod gesture_os_control;

use gesture_os_control::GestureOsControlApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Жесты",
        options,
        Box::new(|_cc| Ok(Box::new(GestureOsControlApp::default()))),
    )
}
