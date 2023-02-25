use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::Client;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;
use serenity::prelude::GatewayIntents;
use serenity::Error;
use serenity::framework::standard::macros::group;

use std::cell::RefCell;
use std::env;

use crate::bot::commands::ALL_COMMANDS;

#[group]
struct General;

struct Handler;

pub struct Bot {
    client: RefCell<Client>,
}

impl Bot {
    pub async fn new(token: String) -> Result<Bot, Box<dyn std::error::Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let client = Client::builder(token, GatewayIntents::empty())
            .event_handler(Handler)
            .framework(framework)
            .await?;

        let bot = Bot {
            client: RefCell::new(client),
        };

        Ok(bot)
    }

    pub async fn start(&self) -> Result<(), Error> {
        let mut client = self.client.borrow_mut();
        client.start().await
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            if let Some(c) = ALL_COMMANDS
                .iter()
                .find(|&c| c.name() == command.data.name.as_str())
            {
                let content = c.run(&command.data.options);
                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(content))
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
            for command in ALL_COMMANDS.iter() {
                commands = commands.create_application_command(|c| command.register(c));
            }

            commands
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}
