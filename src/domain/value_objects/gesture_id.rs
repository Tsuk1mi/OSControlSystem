#![allow(dead_code)]

/// Идентификатор распознанного жеста (внутренний).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum GestureId {
    #[default]
    None,
    SwipeLeft,
    SwipeRight,
    OpenPalm,
    ClosedFist,
    ThumbUp,
    ThumbDown,
    Pointing,
    Victory,
}

impl GestureId {
    pub fn user_trigger_ru(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::SwipeLeft => Some("взмах влево"),
            Self::SwipeRight => Some("взмах вправо"),
            Self::OpenPalm => Some("открытая ладонь"),
            Self::ClosedFist => Some("кулак"),
            Self::ThumbUp => Some("большой палец вверх"),
            Self::ThumbDown => Some("большой палец вниз"),
            Self::Pointing => Some("указание"),
            Self::Victory => Some("victory / peace"),
        }
    }

    /// Имя для `gesture_bindings.json`.
    pub fn wire_key(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::SwipeLeft => Some("SwipeLeft"),
            Self::SwipeRight => Some("SwipeRight"),
            Self::OpenPalm => Some("OpenPalm"),
            Self::ClosedFist => Some("ClosedFist"),
            Self::ThumbUp => Some("ThumbUp"),
            Self::ThumbDown => Some("ThumbDown"),
            Self::Pointing => Some("Pointing"),
            Self::Victory => Some("Victory"),
        }
    }

    pub fn parse_wire_key(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "SwipeLeft" => Self::SwipeLeft,
            "SwipeRight" => Self::SwipeRight,
            "OpenPalm" => Self::OpenPalm,
            "ClosedFist" => Self::ClosedFist,
            "ThumbUp" => Self::ThumbUp,
            "ThumbDown" => Self::ThumbDown,
            "Pointing" => Self::Pointing,
            "Victory" => Self::Victory,
            _ => return None,
        })
    }

    pub fn from_mediapipe_label(label: &str) -> Option<Self> {
        let normalized = label.trim().to_lowercase();
        match normalized.as_str() {
            "open_palm" | "open palm" | "palm" => Some(Self::OpenPalm),
            "closed_fist" | "closed fist" | "fist" => Some(Self::ClosedFist),
            "thumb_up" | "thumb up" => Some(Self::ThumbUp),
            "thumb_down" | "thumb down" => Some(Self::ThumbDown),
            "pointing_up" | "pointing up" | "pointing" => Some(Self::Pointing),
            "victory" | "peace" => Some(Self::Victory),
            _ => None,
        }
    }

    pub const BINDABLE: &'static [GestureId] = &[
        GestureId::SwipeLeft,
        GestureId::SwipeRight,
        GestureId::OpenPalm,
        GestureId::ClosedFist,
        GestureId::ThumbUp,
        GestureId::ThumbDown,
        GestureId::Pointing,
        GestureId::Victory,
    ];
}
