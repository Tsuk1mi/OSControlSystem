#![allow(dead_code)]

use crate::gesture_os_control::application::ports::output::os_command_port::OsCommandPort;
use crate::gesture_os_control::domain::entities::command::{CommandExecutionResult, OsCommand};

/// Заглушка для Linux: команды жестового конвейера здесь не выполняются.
pub struct LinuxOsAdapter;

impl LinuxOsAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: OsCommand) -> CommandExecutionResult {
        let _ = command;
        CommandExecutionResult {
            ok: false,
            description: "Исполнение команд жестов на Linux пока не реализовано.".to_owned(),
            system_error: None,
        }
    }
}

impl OsCommandPort for LinuxOsAdapter {
    fn execute_command(&self, command: OsCommand) -> CommandExecutionResult {
        self.execute(command)
    }
}
