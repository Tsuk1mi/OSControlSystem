use crate::gesture_os_control::application::ports::output::os_command_port::OsCommandPort;
use crate::gesture_os_control::domain::entities::command::{CommandExecutionResult, OsCommand};

pub struct ExecuteCommandUseCase;

impl ExecuteCommandUseCase {
    pub fn run(port: &dyn OsCommandPort, command: OsCommand) -> CommandExecutionResult {
        port.execute_command(command)
    }
}
