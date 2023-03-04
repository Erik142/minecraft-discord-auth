pub mod bot;
pub mod services;

use services::database;
use services::config::Config;
use std::env;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut cfg = Config::from_env().unwrap();
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Could not retrieve DISCORD_TOKEN");
    let db_connection_pool = database::get_connection_pool(&mut cfg.pg);
    let bot = bot::bot::Bot::new(token, db_connection_pool)
        .await
        .expect("Could not create bot!");

    // start listening for events by starting a single shard
    if let Err(why) = bot.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
