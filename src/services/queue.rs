use async_std::task;
use core::time::Duration;
use futures::channel::mpsc;
use futures::channel::mpsc::{UnboundedReceiver, Sender};
use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use tokio_postgres::{connect, AsyncMessage, Client, Error, NoTls};

pub struct MessageQueue<'a> {
    connection_string: &'a str,
    tx_bot: Sender<AsyncMessage>
}

impl MessageQueue<'_> {
    pub async fn start(&mut self) {
        let (mut client, mut rx) = refresh_connection(&self.connection_string).await;
        client.batch_execute("LISTEN bot_updates").await.unwrap();

        loop {
            if client.is_closed() {
                (client, rx) = refresh_connection(&self.connection_string).await;
                client.batch_execute("LISTEN bot_updates").await.unwrap();
            }

            match rx.try_next() {
                Ok(Some(AsyncMessage::Notification(message))) => {
                    loop {
                        match self.tx_bot.try_send(AsyncMessage::Notification(message.clone())) {
                            Ok(()) => break,
                            Err(e) => {
                                println!("An error occured when forwarding message to bot: {}", e);
                                println!("Trying again...");
                                task::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                },
                Ok(_) => (),
                Err(_) => {
                    task::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    pub async fn new(connection_string: &str, tx_bot: Sender<AsyncMessage>) -> Result<MessageQueue, Error> {
        Ok(MessageQueue { connection_string, tx_bot })
    }
}

async fn refresh_connection(connection_string: &str) -> (Client, UnboundedReceiver<AsyncMessage>) {
    println!("The Postgres connection was unexpectedly closed.");

    loop {
        println!("Trying to re-establish Postgres connection...");
        match connect(connection_string, NoTls).await {
            Ok((client, mut connection)) => {
                let (tx, rx) = mpsc::unbounded();
                let stream = stream::poll_fn(move |cx| connection.poll_message(cx))
                    .map_err(|e| panic!("{}", e));
                let c = stream.forward(tx).map(|r| {
                    println!("{:?}", &r);
                    r.unwrap()
                });
                tokio::spawn(c);
                println!("The Postgres connection has been re-established!");
                return (client, rx);
            }
            Err(e) => {
                println!(
                    "An error occured when trying to re-establish the postgres connection: {}",
                    e
                );
                println!("Sleeping for 5 seconds...");
                task::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
