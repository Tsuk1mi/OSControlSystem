//! Техническая реализация: файлы настроек, поток камеры, бэкенд жестов, вспомогательные сервисы.

pub mod app_settings_io;
pub mod app_state;
pub mod context_rules_io;
pub mod error;
pub mod event_bus;
pub mod gesture_backend;
pub mod gesture_bindings_io;
pub mod gesture_camera_service;
pub mod threading;
