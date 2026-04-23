use std::collections::HashMap;

use crate::gesture_os_control::domain::entities::command::OsCommand;
use crate::gesture_os_control::domain::entities::gesture::AppRunMode;
use crate::gesture_os_control::domain::value_objects::gesture_id::GestureId;

// Карта «жест → команда» с учётом режима приложения.
#[derive(Clone, Debug)]
pub struct GestureCommandMap {
    pub desktop: HashMap<GestureId, OsCommand>,
    pub media: HashMap<GestureId, OsCommand>,
    pub browser: HashMap<GestureId, OsCommand>,
}

impl Default for GestureCommandMap {
    fn default() -> Self {
        Self::app_defaults()
    }
}

impl GestureCommandMap {
    pub fn app_defaults() -> Self {
        let mut desktop = HashMap::new();
        desktop.insert(GestureId::SwipeRight, OsCommand::SwitchNextWindow);
        desktop.insert(GestureId::SwipeLeft, OsCommand::MinimizeAllWindows);
        desktop.insert(GestureId::OpenPalm, OsCommand::LockWorkstation);
        desktop.insert(GestureId::ClosedFist, OsCommand::Mute);
        desktop.insert(GestureId::ThumbUp, OsCommand::VolumeUp);
        desktop.insert(GestureId::ThumbDown, OsCommand::VolumeDown);
        desktop.insert(GestureId::Pointing, OsCommand::OpenMenu);
        desktop.insert(GestureId::Victory, OsCommand::OpenExplorer);

        let mut media = desktop.clone();
        media.insert(GestureId::ClosedFist, OsCommand::PlayPause);
        media.insert(GestureId::SwipeRight, OsCommand::NextDesktop);
        media.insert(GestureId::SwipeLeft, OsCommand::PreviousDesktop);
        media.insert(GestureId::Pointing, OsCommand::VolumeDown);
        media.insert(GestureId::OpenPalm, OsCommand::SwitchAudioOutput);
        media.insert(GestureId::ThumbDown, OsCommand::Mute);
        media.insert(GestureId::Victory, OsCommand::OpenNotepad);

        let mut browser = desktop.clone();
        browser.insert(GestureId::SwipeRight, OsCommand::BrowserNextTab);
        browser.insert(GestureId::SwipeLeft, OsCommand::BrowserPreviousTab);
        browser.insert(GestureId::Pointing, OsCommand::ScrollDown);
        browser.insert(GestureId::ThumbUp, OsCommand::ScrollUp);
        browser.insert(GestureId::ThumbDown, OsCommand::BrowserPreviousTab);
        browser.insert(GestureId::Victory, OsCommand::BrowserNextTab);

        Self {
            desktop,
            media,
            browser,
        }
    }

    pub fn lookup(&self, mode: AppRunMode, gesture: GestureId) -> OsCommand {
        let table = match mode {
            AppRunMode::Desktop => &self.desktop,
            AppRunMode::Media => &self.media,
            AppRunMode::Browser => &self.browser,
        };
        table.get(&gesture).copied().unwrap_or(OsCommand::NoAction)
    }

    pub fn table_mut(&mut self, mode: AppRunMode) -> &mut HashMap<GestureId, OsCommand> {
        match mode {
            AppRunMode::Desktop => &mut self.desktop,
            AppRunMode::Media => &mut self.media,
            AppRunMode::Browser => &mut self.browser,
        }
    }

    pub fn table(&self, mode: AppRunMode) -> &HashMap<GestureId, OsCommand> {
        match mode {
            AppRunMode::Desktop => &self.desktop,
            AppRunMode::Media => &self.media,
            AppRunMode::Browser => &self.browser,
        }
    }

    pub fn set_binding(&mut self, mode: AppRunMode, gesture: GestureId, command: OsCommand) {
        if gesture == GestureId::None {
            return;
        }
        self.table_mut(mode).insert(gesture, command);
    }
}

pub struct CommandMapper {
    map: GestureCommandMap,
}

impl CommandMapper {
    pub fn new(map: GestureCommandMap) -> Self {
        Self { map }
    }

    pub fn with_defaults() -> Self {
        Self::new(GestureCommandMap::default())
    }

    pub fn resolve(&self, mode: AppRunMode, gesture: GestureId) -> OsCommand {
        if gesture == GestureId::None {
            return OsCommand::NoAction;
        }
        self.map.lookup(mode, gesture)
    }
}
