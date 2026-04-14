//! Корень подсистемы gesture-os-control (остальной код лежит рядом в `src/`, подключается через `#[path]`).
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
