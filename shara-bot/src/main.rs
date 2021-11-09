use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::config::Config;
use crate::database::Database;

use crate::dialogue::Dialogue;

use crate::handler::{callback, message};

mod captcha;
mod config;
mod database;
mod dialogue;
mod handler;
mod keyboard;
mod reject;

#[tokio::main]
async fn main() {
    let config = Config::new("config.json")
        .await
        .expect("Failed to initialize config");
    let bot = Bot::new(&config.token).auto_send();
    config
        .initialize_data()
        .await
        .expect("Failed to initialize all global data");
    run(bot).await.expect("Something get wrong with main tasks");
}
type In = DialogueWithCx<AutoSend<Bot>, Message, Dialogue, anyhow::Error>;

async fn run(bot: AutoSend<Bot>) -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting shara_bot...");

    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                let dialogue = dialogue.expect("std::convert::Infallible");
                message::handle_message(cx, dialogue)
                    .await
                    .expect("Something wrong with the bot!")
            },
            Arc::new(Database::global()),
        ))
        .callback_queries_handler(callback::handle_callback_query)
        .dispatch()
        .await;
    Ok(())
}
