use super::command::SlashCommand;
use super::commands::pong::PongCommand;

mod pong;

lazy_static! {
    pub static ref ALL_COMMANDS: Vec<Box<dyn SlashCommand>> = {
        let pong: Box<dyn SlashCommand> = PongCommand::new();
        let mut commands = Vec::new();
        commands.push(pong);
        commands
    };
}
