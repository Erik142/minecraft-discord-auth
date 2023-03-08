use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Activity;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::Client;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;
use serenity::prelude::GatewayIntents;
use serenity::utils::Colour;
use serenity::Error;

use tokio::runtime::Runtime;
use tokio_postgres::{AsyncMessage, NoTls};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use std::cell::RefCell;
use std::{env, thread};
use std::sync::{Arc, Mutex};

use crate::bot::command::SlashCommand;
use crate::bot::commands;
use crate::services::embed::EmbedData;

use futures::channel::mpsc::Receiver;

use super::authentication::AuthenticationHandler;

struct Handler {
    commands: Vec<Arc<Box<dyn SlashCommand + 'static>>>,
}

pub struct Bot {
    authentication_handler: Arc<Mutex<AuthenticationHandler>>,
    client: RefCell<Client>,
}

impl Bot {
    pub async fn new(
        token: String,
        db_connection_pool: Pool<PostgresConnectionManager<NoTls>>,
        queue_receiver: Receiver<AsyncMessage>,
    ) -> Result<Bot, Box<dyn std::error::Error>> {
        let pool = Arc::new(db_connection_pool);
        let framework = StandardFramework::new();
        let handler = Handler {
            commands: commands::get_commands(Arc::clone(&pool)),
        };

        let client = Client::builder(token, GatewayIntents::empty())
            .event_handler(handler)
            .framework(framework)
            .await?;

        let bot = Bot {
            client: RefCell::new(client),
            authentication_handler: Arc::new(Mutex::new(AuthenticationHandler::new(Arc::clone(&pool), queue_receiver)))
        };

        Ok(bot)
    }

    pub async fn start(&self) -> Result<(), Error> {
        let cache_http = Arc::clone(&self.client.borrow().cache_and_http);
        let auth_handler = Arc::clone(&self.authentication_handler.clone());

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            loop {
                let cache_http = Arc::clone(&cache_http.clone());
                let auth_handler = auth_handler.lock().unwrap();
                rt.block_on(auth_handler.handle_authentication_requests(cache_http));
            }
        });

        let mut client = self.client.borrow_mut();
        client.start().await
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            if let Some(c) = self
                .commands
                .iter()
                .find(|&c| c.name() == command.data.name.as_str())
            {
                let content = match c.run(&ctx, &command.data.options).await {
                    Ok(result) => result,
                    Err(e) => {
                        println!(
                            "An error occured when executing the command '{}': {}",
                            c.name(),
                            e
                        );
                        EmbedData {
                            title: Some(c.name()),
                            description: Some("An error occured, please try again".to_string()),
                            colour: Some(Colour::RED),
                        }
                    }
                };

                println!("Responding with content '{:?}'", content);

                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.embed(|mut e| {
                                    if let Some(title) = content.title {
                                        e = e.title(title);
                                    };

                                    if let Some(description) = content.description {
                                        e = e.description(description);
                                    };

                                    if let Some(colour) = content.colour {
                                        e = e.colour(colour);
                                    };

                                    e
                                })
                            })
                    })
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |mut commands| {
            for command in self.commands.iter() {
                commands = commands.create_application_command(|c| {
                    let cmd = Arc::clone(command);
                    cmd.register(c)
                });
            }

            commands
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );

        if let Some(version) = option_env!("CARGO_PKG_VERSION") {
            ctx.set_activity(Activity::playing(version)).await;
        }
    }
}
