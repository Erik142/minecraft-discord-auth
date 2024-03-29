use std::sync::Arc;

use super::command::SlashCommand;
use super::commands::pong::PongCommand;
use super::commands::register::RegisterCommand;
use super::commands::unregister::UnregisterCommand;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

mod pong;
mod register;
mod unregister;

pub fn get_commands<'a>(
    db_connection_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
) -> Vec<Arc<Box<dyn SlashCommand + 'static>>> {
    let register_pool = Arc::clone(&db_connection_pool);
    let unregister_pool = Arc::clone(&db_connection_pool);
    let pong: Arc<Box<dyn SlashCommand + 'static>> = Arc::new(PongCommand::new());
    let register: Arc<Box<dyn SlashCommand + 'static>> =
        Arc::new(RegisterCommand::new(register_pool));
    let unregister: Arc<Box<dyn SlashCommand + 'static>> =
        Arc::new(UnregisterCommand::new(unregister_pool));
    let mut commands = Vec::new();
    commands.push(pong);
    commands.push(register);
    commands.push(unregister);
    commands
}
