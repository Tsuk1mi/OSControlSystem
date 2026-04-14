/// Абстрактная команда предметной области (по ТЗ + расширения под текущее приложение).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OsCommand {
    VolumeUp,
    VolumeDown,
    NextDesktop,
    PreviousDesktop,
    ScrollUp,
    ScrollDown,
    PlayPause,
    Mute,
    OpenMenu,
    NoAction,
    SwitchNextWindow,
    LockWorkstation,
    MinimizeAllWindows,
    BrowserNextTab,
    BrowserPreviousTab,
    BrightnessUp,
    BrightnessDown,
    SwitchAudioOutput,
    OpenNotepad,
    OpenExplorer,
    ShutdownComputer,
}

impl OsCommand {
    pub const ALL: &'static [OsCommand] = &[
        OsCommand::NoAction,
        OsCommand::VolumeUp,
        OsCommand::VolumeDown,
        OsCommand::NextDesktop,
        OsCommand::PreviousDesktop,
        OsCommand::ScrollUp,
        OsCommand::ScrollDown,
        OsCommand::PlayPause,
        OsCommand::Mute,
        OsCommand::OpenMenu,
        OsCommand::SwitchNextWindow,
        OsCommand::LockWorkstation,
        OsCommand::MinimizeAllWindows,
        OsCommand::BrowserNextTab,
        OsCommand::BrowserPreviousTab,
        OsCommand::BrightnessUp,
        OsCommand::BrightnessDown,
        OsCommand::SwitchAudioOutput,
        OsCommand::OpenNotepad,
        OsCommand::OpenExplorer,
        OsCommand::ShutdownComputer,
    ];

    pub fn label_ru(self) -> &'static str {
        match self {
            OsCommand::VolumeUp => "Громкость +",
            OsCommand::VolumeDown => "Громкость −",
            OsCommand::NextDesktop => "Следующий вирт. стол",
            OsCommand::PreviousDesktop => "Предыдущий вирт. стол",
            OsCommand::ScrollUp => "Прокрутка вверх",
            OsCommand::ScrollDown => "Прокрутка вниз",
            OsCommand::PlayPause => "Пауза / воспроизведение",
            OsCommand::Mute => "Без звука",
            OsCommand::OpenMenu => "Меню Win+X",
            OsCommand::NoAction => "Нет действия",
            OsCommand::SwitchNextWindow => "Следующее окно (Alt+Tab)",
            OsCommand::LockWorkstation => "Заблокировать ПК",
            OsCommand::MinimizeAllWindows => "Свернуть всё (Win+D)",
            OsCommand::BrowserNextTab => "Браузер: следующая вкладка",
            OsCommand::BrowserPreviousTab => "Браузер: предыдущая вкладка",
            OsCommand::BrightnessUp => "Яркость +",
            OsCommand::BrightnessDown => "Яркость −",
            OsCommand::SwitchAudioOutput => "Сменить аудиовыход",
            OsCommand::OpenNotepad => "Блокнот",
            OsCommand::OpenExplorer => "Проводник",
            OsCommand::ShutdownComputer => "Выключение ПК (2×)",
        }
    }

    /// Имя для `gesture_bindings.json` (совпадает с `Debug`).
    pub fn wire_key(self) -> &'static str {
        match self {
            OsCommand::VolumeUp => "VolumeUp",
            OsCommand::VolumeDown => "VolumeDown",
            OsCommand::NextDesktop => "NextDesktop",
            OsCommand::PreviousDesktop => "PreviousDesktop",
            OsCommand::ScrollUp => "ScrollUp",
            OsCommand::ScrollDown => "ScrollDown",
            OsCommand::PlayPause => "PlayPause",
            OsCommand::Mute => "Mute",
            OsCommand::OpenMenu => "OpenMenu",
            OsCommand::NoAction => "NoAction",
            OsCommand::SwitchNextWindow => "SwitchNextWindow",
            OsCommand::LockWorkstation => "LockWorkstation",
            OsCommand::MinimizeAllWindows => "MinimizeAllWindows",
            OsCommand::BrowserNextTab => "BrowserNextTab",
            OsCommand::BrowserPreviousTab => "BrowserPreviousTab",
            OsCommand::BrightnessUp => "BrightnessUp",
            OsCommand::BrightnessDown => "BrightnessDown",
            OsCommand::SwitchAudioOutput => "SwitchAudioOutput",
            OsCommand::OpenNotepad => "OpenNotepad",
            OsCommand::OpenExplorer => "OpenExplorer",
            OsCommand::ShutdownComputer => "ShutdownComputer",
        }
    }

    pub fn parse_wire_key(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "VolumeUp" => OsCommand::VolumeUp,
            "VolumeDown" => OsCommand::VolumeDown,
            "NextDesktop" => OsCommand::NextDesktop,
            "PreviousDesktop" => OsCommand::PreviousDesktop,
            "ScrollUp" => OsCommand::ScrollUp,
            "ScrollDown" => OsCommand::ScrollDown,
            "PlayPause" => OsCommand::PlayPause,
            "Mute" => OsCommand::Mute,
            "OpenMenu" => OsCommand::OpenMenu,
            "NoAction" => OsCommand::NoAction,
            "SwitchNextWindow" => OsCommand::SwitchNextWindow,
            "LockWorkstation" => OsCommand::LockWorkstation,
            "MinimizeAllWindows" => OsCommand::MinimizeAllWindows,
            "BrowserNextTab" => OsCommand::BrowserNextTab,
            "BrowserPreviousTab" => OsCommand::BrowserPreviousTab,
            "BrightnessUp" => OsCommand::BrightnessUp,
            "BrightnessDown" => OsCommand::BrightnessDown,
            "SwitchAudioOutput" => OsCommand::SwitchAudioOutput,
            "OpenNotepad" => OsCommand::OpenNotepad,
            "OpenExplorer" => OsCommand::OpenExplorer,
            "ShutdownComputer" => OsCommand::ShutdownComputer,
            _ => return None,
        })
    }
}

/// Результат выполнения команды в ОС.
#[derive(Clone, Debug)]
pub struct CommandExecutionResult {
    pub ok: bool,
    pub description: String,
    pub system_error: Option<String>,
}
