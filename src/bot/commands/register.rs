use std::sync::Arc;

use crate::services::database::Database;
use crate::services::database::DatabaseError;
use crate::services::embed::EmbedData;

use super::super::command::SlashCommand;
use async_trait::async_trait;
use rand::{distributions::Alphanumeric, Rng};
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use serenity::utils::Colour;

use std::error::Error;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

static NAME: &str = "register";
static DESCRIPTION: &str = "Register yourself to the Minecraft server";

pub struct RegisterCommand {
    db_connection_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

#[async_trait]
impl SlashCommand for RegisterCommand {
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
                match database.get_minecraft_user(&discord_user.id.to_string()).await {
                    Ok(_) => return Ok(EmbedData {
                        title: Some("Minecraft registration".to_string()),
                        description: Some("You are already registered on the Minecraft server. Please unregister before trying to register again.".to_string()),
                        colour: Some(Colour::RED),
                    }),
                    Err(DatabaseError::MissingMinecraftId(_)) => {
                        let result = database.get_reg_code(&discord_user.id.to_string()).await;
                        match result {
                            Ok(reg_code) => {
                                return Ok(EmbedData{
                                    title: Some("Minecraft registration".to_string()),
                                    description: Some(format!("You have a pending registration status. To complete the registration process, please open Minecraft, click on Multiplayer and join the server\n\n```\nminecraft.wahlberger.dev\n```\n\nOnce joined, enter the following command in minecraft:\n\n```\n/register {}\n```\n\nto link your Minecraft account to your Discord account.", reg_code).to_string()),
                                    colour: Some(Colour::RED),
                                })
                            },
                            Err(e) => return Err(Box::new(e))
                        }
                    },
                    Err(e) => return Err(Box::new(e)),
                }
            },
            Err(DatabaseError::PlayerNotRegistered(_)) => {
                let reg_code: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();

                database.add_player(&discord_user.id.to_string(), &reg_code).await?;
                return Ok(EmbedData {
                        title: Some("Minecraft registration".to_string()),
                        description: Some(format!("There we go! I have added a registration request for you! To complete the registration process, please open Minecraft, click on Multiplayer and join the server\n\n```\nminecraft.wahlberger.dev\n```\n\nOnce joined, enter the following command in minecraft:\n\n```\n/register {}\n```\n\nto link your Minecraft account to your Discord account.", reg_code)),
                        colour: Some(Colour::DARK_GREEN),
                    })
            },
            Err(e) => Err(Box::new(e))
        }
    }
}

impl RegisterCommand {
    pub fn new(db_connection_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>) -> Box<dyn SlashCommand + 'static> {
        let pool = Arc::clone(&db_connection_pool);
        Box::new(RegisterCommand {
            db_connection_pool: pool,
        })
    }
}
