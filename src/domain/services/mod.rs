//! Сервисы предметной области: распознавание, фильтрация, безопасность, контекст.

pub mod command_mapper;
pub mod context_resolver;
pub mod face_exclusion;
pub mod gesture_classifier;
#[cfg(feature = "opencv")]
pub mod opencv_skin_mask;
pub mod safety_guard;
pub mod temporal_filter;
