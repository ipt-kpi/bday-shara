use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::database::Database;
use crate::keyboard::Keyboard;

pub async fn handle_callback_query(rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move { handle_callback(cx).await })
        .await;
}

async fn handle_callback(cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>) {
    let query = &cx.update;
    if let Some(message) = &query.message {
        let data = query
            .data
            .as_ref()
            .map(|text| text.parse::<i32>().ok());
        if let Some(id) = data.flatten() {
            match Database::global().mark_prize(query.from.id, id).await {
                Ok(_) => {
                    match Database::global().get_prizes(query.from.id).await {
                        Ok(prizes) => {
                            let mut message = cx.requester
                                .edit_message_text(query.from.id, message.id, format!("{}", prizes))
                                .parse_mode(ParseMode::Html);
                            if let Some(keyboard) = Keyboard::global().get_prize_keyboard(prizes).await {
                                message = message.reply_markup(keyboard)
                            }
                            message.send().await;
                        },
                        Err(error) => {
                            cx.requester.send_message(query.from.id, "Виникла помилка при обробці вибору!").await;
                            log::error!(
                            "Error while get prize list (user_id: {}): {}",
                            query.from.id,
                            error
                        );
                        }
                    }
                }
                Err(error) => {
                    cx.requester.send_message(query.from.id, "Виникла помилка при обробці вибору!").await;
                    log::error!(
                    "Error while mark prize (user_id: {}, prize_id: {}): {}",
                    query.from.id,
                    id,
                    error
                );
                }
            }
        }
    }
}
