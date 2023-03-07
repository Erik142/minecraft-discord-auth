use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio_postgres::Config;
use tokio_postgres::NoTls;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Could not insert {data:?} (Error: {why:?})")]
    InsertError { data: String, why: String },
    #[error("Could not delete {data:?} (Error: {why:?})")]
    DeleteError { data: String, why: String },
    #[error("Could not select {data:?} (Error: {why:?})")]
    SelectError { data: String, why: String },
    #[error("Could not find a Discord user with the Minecraft user name '{0}'")]
    MissingDiscordId(String),
    #[error("Could not find a Minecraft player for Discord user with the id '{0}'")]
    MissingMinecraftId(String),
    #[error("Could not find a registration code for Discord user with the id '{0}'")]
    MissingRegistration(String),
    #[error("The Discord user with the id '{0}' has not been registered on the Minecraft server")]
    PlayerNotRegistered(String),
    #[error(
        "The Discord user with the id '{0}' has not been authenticated on the Minecraft server"
    )]
    PlayerNotAuthenticated(String),
}

pub struct Database {
    pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

impl Database {
    pub async fn add_player(&self, discord_id: &str, reg_code: &str) -> Result<(), DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let result = connection
            .execute(
                "INSERT INTO Players(discordname, registrationcode) VALUES($1, $2)",
                &[&discord_id, &reg_code],
            )
            .await;

        match result {
            Ok(r) if r == 0 => Err(DatabaseError::InsertError {
                data: "player".to_string(),
                why: "0 rows inserted!".to_string(),
            }),
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::InsertError {
                data: "player".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn add_player_auth(
        &self,
        discord_id: &str,
        auth_request_id: &str,
    ) -> Result<(), DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let result = connection
            .execute(
                "INSERT INTO PlayerAuthentications(discordname, authrequestid) VALUES($1, $2)",
                &[&discord_id, &auth_request_id],
            )
            .await;

        match result {
            Ok(r) if r == 0 => Err(DatabaseError::InsertError {
                data: "Player authentication".to_string(),
                why: "0 rows inserted!".to_string(),
            }),
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::InsertError {
                data: "Player authentication".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn delete_player(&self, discord_id: &str) -> Result<(), DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let result = connection
            .execute("DELETE FROM Players WHERE discordname=$1", &[&discord_id])
            .await;

        match result {
            Ok(r) if r == 0 => Err(DatabaseError::MissingDiscordId(discord_id.to_string())),
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::DeleteError {
                data: "Discord id".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn delete_player_auth(&self, discord_id: &str) -> Result<(), DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let result = connection
            .execute(
                "DELETE FROM PlayerAuthentications WHERE discordname=$1",
                &[&discord_id],
            )
            .await;

        match result {
            Ok(r) if r == 0 => Err(DatabaseError::MissingDiscordId(discord_id.to_string())),
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::DeleteError {
                data: "Player authentication".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn get_discord_id(&self, minecraft_user_id: &str) -> Result<String, DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let row = connection
            .query_one(
                "SELECT discordname FROM Players WHERE minecraftname=$1",
                &[&minecraft_user_id],
            )
            .await;

        match row {
            Ok(r) if r.is_empty() => Err(DatabaseError::MissingDiscordId(
                minecraft_user_id.to_string(),
            )),
            Ok(r) => {
                let result: Result<Option<String>, tokio_postgres::Error> =
                    r.try_get("discordname");

                match result {
                    Ok(Some(discord_id)) => Ok(discord_id),
                    Ok(None) => Err(DatabaseError::MissingDiscordId(
                        minecraft_user_id.to_string(),
                    )),
                    Err(e) => Err(DatabaseError::SelectError {
                        data: "Discord id".to_string(),
                        why: e.to_string(),
                    }),
                }
            }

            Err(e) => Err(DatabaseError::SelectError {
                data: "Discord id".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn get_minecraft_user(&self, discord_id: &str) -> Result<String, DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let row = connection
            .query_one(
                "SELECT minecraftname FROM Players WHERE discordname=$1",
                &[&discord_id],
            )
            .await;

        return match row {
            Err(e) => Err(DatabaseError::SelectError {
                data: "Minecraft user name".to_string(),
                why: e.to_string(),
            }),
            Ok(r) if r.is_empty() => Err(DatabaseError::MissingMinecraftId(discord_id.to_string())),
            Ok(r) => {
                let result: Result<Option<String>, tokio_postgres::Error> =
                    r.try_get("minecraftname");
                match result {
                    Ok(Some(minecraft_name)) => Ok(minecraft_name),
                    Ok(None) | Err(_) => {
                        Err(DatabaseError::MissingMinecraftId(discord_id.to_string()))
                    }
                }
            }
        };
    }

    pub async fn get_reg_code(&self, discord_id: &str) -> Result<String, DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let row = connection
            .query_one(
                "SELECT registrationcode FROM Players WHERE discordname=$1",
                &[&discord_id],
            )
            .await;

        match row {
            Ok(r) if r.is_empty() => {
                Err(DatabaseError::MissingRegistration(discord_id.to_string()))
            }
            Ok(r) => {
                let result: Result<Option<String>, tokio_postgres::Error> =
                    r.try_get("registrationcode");

                match result {
                    Ok(Some(reg_code)) => Ok(reg_code),
                    Ok(None) => Err(DatabaseError::MissingRegistration(discord_id.to_string())),
                    Err(e) => Err(DatabaseError::SelectError {
                        data: "registration code".to_string(),
                        why: e.to_string(),
                    }),
                }
            }
            Err(e) => Err(DatabaseError::SelectError {
                data: "Discord id".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn is_player_authenticated(
        &self,
        discord_id: &str,
        ip_address: &str,
    ) -> Result<bool, DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let row =
            connection
            .query_one(
                "SELECT COUNT(*) FROM AuthenticatedPlayers INNER JOIN AuthenticationRequests ON (AuthenticatedPlayers.authrequestid=AuthenticationRequests.id) WHERE AuthenticatedPlayers.discordname=$1 AND AuthenticationRequests.ipaddress=$2",
                &[&discord_id, &ip_address],
            )
            .await;

        match row {
            Ok(r) if r.is_empty() || r.get::<usize, i64>(0) == 0 => Err(
                DatabaseError::PlayerNotAuthenticated(discord_id.to_string()),
            ),
            Ok(_) => Ok(true),
            Err(e) => Err(DatabaseError::SelectError {
                data: "Discord id".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn is_player_registered(&self, discord_id: &str) -> Result<bool, DatabaseError> {
        let pool = Arc::clone(&self.pool);
        let connection = pool.get().await.unwrap();
        let row = connection
            .query_one(
                "SELECT COUNT(*) FROM Players WHERE discordname=$1",
                &[&discord_id],
            )
            .await;

        match row {
            Ok(r) if r.is_empty() || r.get::<usize, i64>(0) == 0 => {
                Err(DatabaseError::PlayerNotRegistered(discord_id.to_string()))
            }
            Ok(_) => Ok(true),
            Err(e) => Err(DatabaseError::SelectError {
                data: "Discord id".to_string(),
                why: e.to_string(),
            }),
        }
    }

    pub async fn new(
        pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
    ) -> Result<Database, Box<dyn Error>> {
        Ok(Database {
            pool: Arc::clone(&pool),
        })
    }
}

pub async fn get_connection_pool(
    connection_string: &str,
) -> Pool<PostgresConnectionManager<NoTls>> {
    let manager = bb8_postgres::PostgresConnectionManager::new(
        Config::from_str(connection_string).unwrap(),
        NoTls,
    );
    let pool = bb8::Pool::builder()
        .max_size(20)
        .build(manager)
        .await
        .unwrap();

    return pool;
}
