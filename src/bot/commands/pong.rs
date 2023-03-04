use crate::services::embed::EmbedData;

use super::super::command::SlashCommand;
use async_trait::async_trait;
use serenity::{
    model::prelude::interaction::application_command::CommandDataOption, prelude::Context,
    utils::Colour,
};
use std::error::Error;

static NAME: &str = "ping";
static DESCRIPTION: &str = "A ping command";

pub struct PongCommand;

#[async_trait]
impl SlashCommand for PongCommand {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn description(&self) -> String {
        DESCRIPTION.to_string()
    }

    async fn run(&self, _: &Context, _: &[CommandDataOption]) -> Result<EmbedData, Box<dyn Error>> {
        Ok(EmbedData {
            title: Some("Ping pong".to_string()),
            description: Some("Pong!".to_string()),
            colour: Some(Colour::DARK_GREEN),
        })
    }
}

impl<'a> PongCommand {
    pub fn new() -> Box<dyn SlashCommand + 'static> {
        Box::new(PongCommand {})
    }
}
