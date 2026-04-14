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
    Pointing,
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
            Self::Pointing => Some("указание"),
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
            Self::Pointing => Some("Pointing"),
        }
    }

    pub fn parse_wire_key(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "SwipeLeft" => Self::SwipeLeft,
            "SwipeRight" => Self::SwipeRight,
            "OpenPalm" => Self::OpenPalm,
            "ClosedFist" => Self::ClosedFist,
            "ThumbUp" => Self::ThumbUp,
            "Pointing" => Self::Pointing,
            _ => return None,
        })
    }

    pub const BINDABLE: &'static [GestureId] = &[
        GestureId::SwipeLeft,
        GestureId::SwipeRight,
        GestureId::OpenPalm,
        GestureId::ClosedFist,
        GestureId::ThumbUp,
        GestureId::Pointing,
    ];
}
