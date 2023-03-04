use std::sync::Arc;

use crate::services::database::Database;
use crate::services::database::DatabaseError;
use crate::services::embed::EmbedData;

use super::super::command::SlashCommand;
use async_trait::async_trait;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use serenity::utils::Colour;

use std::error::Error;

use deadpool_postgres::Pool;

static NAME: &str = "unregister";
static DESCRIPTION: &str = "Unregister yourself from the Minecraft server";

pub struct UnregisterCommand {
    db_connection_pool: Arc<Pool>,
}

#[async_trait]
impl SlashCommand for UnregisterCommand {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn description(&self) -> String {
        DESCRIPTION.to_string()
    }

    async fn run(
        &self,
        ctx: &Context,
        _: &[CommandDataOption],
    ) -> Result<EmbedData, Box<dyn Error>> {
        let pool = Arc::clone(&self.db_connection_pool);
        let database = Database::new(pool).await?;

        let discord_user = ctx.http.get_current_user().await?;
        let is_player_registered = database
            .is_player_registered(&discord_user.id.to_string())
            .await;
        match is_player_registered {
            Ok(_) => {
                match database.delete_player(&discord_user.id.to_string()).await {
                    Ok(_) => return Ok(EmbedData {
                        title: Some("Minecraft unregistration".to_string()),
                        description: Some("You have been successfully unregistered from the Minecraft server".to_string()),
                        colour: Some(Colour::DARK_GREEN),
                    }),
                    Err(e) => return Err(Box::new(e)),
                }
            }
            Err(DatabaseError::PlayerNotRegistered(_)) => {
                return Ok(EmbedData {
                        title: Some("Minecraft unregistration".to_string()),
                        description: Some("You are not registered on the Minecraft server.".to_string()),
                        colour: Some(Colour::RED),
                    });
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl UnregisterCommand {
    pub fn new(db_connection_pool: Arc<Pool>) -> Box<dyn SlashCommand + 'static> {
        let pool = Arc::clone(&db_connection_pool);
        Box::new(UnregisterCommand {
            db_connection_pool: pool,
        })
    }
}
