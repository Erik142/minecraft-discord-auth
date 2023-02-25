use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub trait SlashCommand: Send + Sync {
    fn name(&self) -> String;
    fn register<'a>(&'a self, command: &'a mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
    fn run(&self, options: &[CommandDataOption]) -> String;
}
