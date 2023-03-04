use std::sync::Arc;

use deadpool_postgres::Pool;

use super::command::SlashCommand;
use super::commands::pong::PongCommand;
use super::commands::register::RegisterCommand;

mod pong;
mod register;

pub fn get_commands<'a>(
    db_connection_pool: Arc<Pool>,
) -> Vec<Arc<Box<dyn SlashCommand + 'static>>> {
    let db_connection_pool = Arc::clone(&db_connection_pool);
    let pong: Arc<Box<dyn SlashCommand + 'static>> = Arc::new(PongCommand::new());
    let register: Arc<Box<dyn SlashCommand + 'static>> =
        Arc::new(RegisterCommand::new(db_connection_pool));
    let mut commands = Vec::new();
    commands.push(pong);
    commands.push(register);
    commands
}
