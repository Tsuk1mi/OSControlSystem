use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GestureBackendKind {
    Classic,
    #[default]
    MediaPipe,
}

impl GestureBackendKind {
    pub fn label_ru(self) -> &'static str {
        match self {
            Self::Classic => "Классический CV",
            Self::MediaPipe => "MediaPipe",
        }
    }

    pub fn description_ru(self) -> &'static str {
        match self {
            Self::Classic => "Лёгкий встроенный детектор ладони и геометрический классификатор.",
            Self::MediaPipe => {
                "MediaPipe Hands через Python helper (нужны Python 3 и пакет mediapipe). Иначе — Classic fallback."
            }
        }
    }
}
