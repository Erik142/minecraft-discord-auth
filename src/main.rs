#[macro_use]
extern crate lazy_static;

pub mod bot;

use std::env;

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let bot = bot::bot::Bot::new(token).await.expect("Could not create bot!");

    // start listening for events by starting a single shard
    if let Err(why) = bot.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
