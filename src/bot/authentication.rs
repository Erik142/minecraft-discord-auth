use std::{sync::{Arc, Mutex}, time::{Instant, Duration}};

use async_std::task;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use futures::channel::mpsc::Receiver;
use serenity::{http::CacheHttp, CacheAndHttp, utils::Colour};
use tokio_postgres::{AsyncMessage, NoTls, Notification};

use crate::services::database::Database;

pub struct AuthenticationHandler {
    db_connection_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
    queue_rx: Arc<Mutex<Receiver<AsyncMessage>>>,
}

impl AuthenticationHandler {
    pub async fn handle_authentication_requests(&self, cache_http: Arc<CacheAndHttp>) {
        loop {
            let mut queue_rx = self.queue_rx.lock().unwrap();
            match &queue_rx.try_next() {
                Ok(Some(AsyncMessage::Notification(message))) => self.process_authentication(message, Arc::clone(&cache_http)).await,
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }

    async fn process_authentication(&self, message: &Notification, cache_http: Arc<CacheAndHttp>) {
        let pool = Arc::clone(&self.db_connection_pool);
        let db = Database::new(pool).await.unwrap();

        let request_id = message.payload().parse::<i32>().unwrap();
        let minecraft_user = db.get_authentication_request_user(&request_id).await;
        let ip_address = db.get_authentication_request_ip_address(&request_id).await;

        if let Err(e) = &minecraft_user {
            println!("Could not process authentication request. User could not be retrieved from Postgres: {}", e);
        }

        if let Err(e) = &ip_address {
            println!("Could not process authentication request. Ip address could not be retrieved from Postgres: {}", e);
        }

        if minecraft_user.is_err() || ip_address.is_err() {
            return
        }

        let minecraft_user = minecraft_user.unwrap();
        let ip_address = ip_address.unwrap();

        let discord_user = db.get_discord_id(&minecraft_user).await;

        if let Err(e) = &discord_user {
            println!("Could not process authentication request. Discord user could not be retrieved from Postgres: {}", e);
            return
        }

        let discord_user = discord_user.unwrap();
        let discord_user = discord_user.parse::<u64>();

        if let Err(e) = &discord_user {
            println!("Could not process authentication request. Discord user could not be parsed: {}", e);
            return
        }

        let discord_user = discord_user.unwrap();
        let http = cache_http.http();
        let user = http.get_user(discord_user).await;

        if let Err(e) = &user {
            println!("Could not process authentication request. User could not be fetched from Discord servers: {}", e);
            return
        }

        let user = user.unwrap();
        let channel = user.create_dm_channel(&cache_http).await;

        if let Err(e) = &channel {
            println!("Could not process authentication request. Could not create a DM channel with the user {}: {}", user.name, e);
            return
        }

        let channel = channel.unwrap();
        let message = channel.send_message(http, |c| c.add_embed(|e| e.title("Minecraft login").description(format!("The Minecraft user {} tried to login on the Minecraft server. Was it you?", minecraft_user)))).await;

        if let Err(e) = message {
            println!("Could not process authentication request. Could not send a DM to the user {}: {}", user.name, e);
            return
        }

        let message = message.unwrap();
        let yes_reaction = message.react(&cache_http, '✅').await;

        if let Err(e) = yes_reaction {
            println!("Could not process authentication request. Could not react on DM sent to the user {}: {}", user.name, e);
            return
        }

        let yes_reaction = yes_reaction.unwrap();
        let no_reaction = message.react(&cache_http, '❌').await;

        if let Err(e) = no_reaction {
            println!("Could not process authentication request. Could not react on DM sent to the user {}: {}", user.name, e);
            return
        }

        let no_reaction = no_reaction.unwrap();
        let mut user_reaction = None;
        let start_time = Instant::now();
        while start_time.elapsed().as_secs() < Duration::from_secs(30).as_secs() {
            let yes_reactions = message.reaction_users(&http, yes_reaction.emoji.clone(), Option::None, Option::None).await;
            let no_reactions = message.reaction_users(&http, no_reaction.emoji.clone(), Option::None, Option::None).await;

            if let Err(e) = &yes_reactions {
                println!("Could not process authentication request. Could not read 'yes' reaction on DM sent to the user {}: {}", user.name, e);
                return
            }

            if let Err(e) = &no_reactions {
                println!("Could not process authentication request. Could not read 'no' reaction on DM sent to the user {}: {}", user.name, e);
                return
            }

            let yes_reactions = yes_reactions.unwrap();
            let no_reactions = no_reactions.unwrap();

            if yes_reactions.len() > 1 {
                println!("User {} reacted with 'yes'!", user.name);
                user_reaction = Some(&yes_reaction);
                break;
            }

            if no_reactions.len() > 1 {
                println!("User {} reacted with 'no'!", user.name);
                user_reaction = Some(&no_reaction);
                break;
            }

            println!("User {} has not reacted yet...", user.name);
            task::sleep(Duration::from_secs(1)).await;
        }

        let start_time = Instant::now();
        let mut message_confirmation = None;

        if let Some(user_reaction) = user_reaction {
            if user_reaction.emoji == yes_reaction.emoji {
                let is_authenticated = db.is_player_authenticated(&discord_user.to_string(), &ip_address).await;

                if let Err(e) = &is_authenticated {
                    println!("Could not process authentication request. Could not retrieve current authentication status for user {}: {}", user.name, e);
                    return
                }

                let is_authenticated = is_authenticated.unwrap();

                if is_authenticated {
                    message_confirmation = Some(channel.send_message(http, |c| c.add_embed(|e| e.title("Minecraft login").description("You are already logged in on the Minecraft server.").color(Colour::RED))).await);
                } else {
                    let _ = db.delete_player_auth(&discord_user.to_string()).await;
                    let add_auth_result = db.add_player_auth(&discord_user.to_string(), &request_id).await;

                    if let Err(e) = add_auth_result {
                        println!("Could not process authentication request. Could not add new authentication for user {}: {}", user.name, e);
                        message_confirmation = Some(channel.send_message(http, |c| c.add_embed(|e| e.title("Minecraft login").description("An internal error occured. Please try again or contact Mr. Erkaberka if this problem persists!").color(Colour::RED))).await);
                    } else {
                        message_confirmation = Some(channel.send_message(http, |c| c.add_embed(|e| e.title("Minecraft login").description("The login request has been approved. You can now join the protected Minecraft servers for the next 30 minutes.").color(Colour::DARK_GREEN))).await);
                    }
                }
            } else {
                message_confirmation = Some(channel.send_message(http, |c| c.add_embed(|e| e.title("Minecraft login").description("The login request has been denied. Contact the Discord moderators if you keep receiving login requests from me.").color(Colour::RED))).await);
            }
        }


        while start_time.elapsed().as_secs() < Duration::from_secs(30).as_secs() {
            task::sleep(Duration::from_secs(1)).await;
        }

        if let Err(e) = message.delete(&cache_http).await {
            println!("An error occured when deleting initial DM to user {}: {}", user.name, e);
        }

        if let Some(message_confirmation) = message_confirmation {
            if let Err(e) = message_confirmation {
                println!("Could not process authentication request. Could not send DM confirmation to user {}: {}", user.name, e);
                return
            }

            let message_confirmation = message_confirmation.unwrap();

            if let Err(e) = message_confirmation.delete(&cache_http).await {
                println!("An error occured when deleting DM confirmation to user {}: {}", user.name, e);
            }
        }
    }

    pub fn new(
        db_connection_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
        queue_rx: Receiver<AsyncMessage>
        ) -> AuthenticationHandler {
        AuthenticationHandler { db_connection_pool, queue_rx: Arc::new(Mutex::new(queue_rx)) }
    }
}
