use teloxide::prelude::*;

use crate::database::Database;
use crate::dialogue::states::StartState;
use crate::dialogue::Dialogue;
use crate::keyboard::Keyboard;

pub async fn handle_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    match cx.update.text().map(ToOwned::to_owned) {
        None => {
            cx.answer("Відправ мені текстове повідомлення").await?;
            next(dialogue)
        }
        Some(ans) => match ans.as_str() {
            "/start" => {
                cx.answer(
                    "Щоб продовжити роботу з ботом, погодьтеся зі збором та обробкою персональних даних у вигляді ПІБ та групи",
                )
                    .reply_markup(Keyboard::global().get_agree_keyboard())
                    .send()
                    .await?;
                if !dialogue.is_start() {
                    match Database::global()
                        .refresh_user_state(cx.update.chat_id())
                        .await
                    {
                        Ok(_) => next(Dialogue::Start(StartState)),
                        Err(error) => {
                            cx.answer("Не вдалося перезапустити бота").await?;
                            log::error!("Database error: {}", error);
                            next(dialogue)
                        }
                    }
                } else {
                    next(dialogue)
                }
            }
            ans => dialogue.react(cx, ans.to_string()).await,
        },
    }
}
