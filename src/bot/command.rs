use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use std::error::Error;

use crate::services::embed::EmbedData;

#[async_trait]
pub trait SlashCommand: Send + Sync {
    fn description(&self) -> String;
    fn name(&self) -> String;
    fn register<'a>(
        &self,
        command: &'a mut CreateApplicationCommand,
    ) -> &'a mut CreateApplicationCommand {
        command.name(self.name()).description(self.description())
    }
    async fn run(
        &self,
        ctx: &Context,
        options: &[CommandDataOption],
    ) -> Result<EmbedData, Box<dyn Error>>;
}
