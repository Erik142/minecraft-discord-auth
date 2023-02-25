use super::super::command::SlashCommand;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

static NAME: &str = "ping";

pub struct PongCommand;

impl SlashCommand for PongCommand {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn register<'a>(
        &'a self,
        command: &'a mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name(NAME.to_string())
            .description(String::from("A ping command"))
    }

    fn run(&self, _: &[CommandDataOption]) -> String {
        "pong".to_string()
    }
}

impl PongCommand {
    pub fn new() -> Box<PongCommand> {
        Box::new(PongCommand{})
    }
}
