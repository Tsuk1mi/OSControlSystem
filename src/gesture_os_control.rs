//! Подсистема управления жестами: домен, приложение, инфраструктура, адаптеры, UI.
//! Все внутренние модули лежат в `src/` и подключаются здесь через `#[path]`, т.к. отдельный
//! crate не вынесен — так упрощена сборка одного бинарника.
#![allow(dead_code)]

#[path = "adapters/mod.rs"]
pub mod adapters;
#[path = "application/mod.rs"]
pub mod application;
#[path = "domain/mod.rs"]
pub mod domain;
#[path = "infrastructure/mod.rs"]
pub mod infrastructure;
#[path = "presentation/mod.rs"]
pub mod presentation;

pub use domain::entities::gesture::{AppRunMode, PipelineGestureStats};
pub use presentation::GestureOsControlApp;
