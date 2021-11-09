use crate::database::Database;
use itertools::Itertools;
use teloxide::prelude::*;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn handle_callback_query(rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move { handle_callback(cx).await })
        .await;
}

async fn handle_callback(cx: UpdateWithCx<AutoSend<Bot>, CallbackQuery>) {
    let query = &cx.update;
    let text = query
        .message
        .as_ref()
        .and_then(|msg| msg.text())
        .map(|text| text.split("\n").map(str::to_string).collect::<Vec<_>>());
    let data = query
        .data
        .as_ref()
        .map(|text| text.split(":"))
        .and_then(|mut text| text.next_tuple())
        .map(|(id, number)| (id.parse::<i32>().ok(), number.parse::<usize>().ok()));
    let prize = match data {
        Some((Some(id), Some(number))) => Some((id, number)),
        _ => None,
    };
    if let Some((id, number)) = prize {
        match Database::global().mark_prize(query.from.id, id).await {
            Ok(_) => {
                let text = text.map(|mut text| {
                    if let Some(row) = text.get_mut(number) {
                        if row.contains("✅") {
                            *row = row.replace("✅", "❌")
                        } else if row.contains("❌") {
                            *row = row.replace("❌", "✅")
                        }
                    }
                    text.join("\n")
                });
                if let Some(text) = text {
                    let message = query.message.as_ref().unwrap();
                    if let Some(markup) = message.reply_markup() {
                        cx.requester
                            .edit_message_text(query.from.id, message.id, text)
                            .reply_markup(markup.clone())
                            .send()
                            .await;
                    }
                }
            }
            Err(error) => {
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
