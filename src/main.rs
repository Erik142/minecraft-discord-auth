pub mod bot;
pub mod services;

use dotenvy::dotenv;
use services::database;
use services::queue::MessageQueue;
use std::env;
use futures::channel::mpsc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Could not retrieve DISCORD_TOKEN");
    let postgres_connection_string =
        env::var("POSTGRES_URL").expect("Could not retrieve POSTGRES_URL");

    let (tx, rx) = mpsc::channel(100);

    let db_connection_pool = database::get_connection_pool(&postgres_connection_string).await;
    tokio::spawn(async move {
        let mut queue = MessageQueue::new(&postgres_connection_string, tx)
            .await
            .unwrap();
        queue.start().await;
    });
    let bot = bot::bot::Bot::new(token, db_connection_pool, rx)
        .await
        .expect("Could not create bot!");

    // start listening for events by starting a single shard
    if let Err(why) = bot.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
