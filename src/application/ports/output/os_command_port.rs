use crate::gesture_os_control::domain::entities::command::{CommandExecutionResult, OsCommand};

pub trait OsCommandPort: Send + Sync {
    fn execute_command(&self, command: OsCommand) -> CommandExecutionResult;
}
