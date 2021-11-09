use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;

use crate::captcha::Captcha;
use crate::dialogue::states::receive_captcha::ReceiveCaptchaState;
use crate::dialogue::Dialogue;
use crate::keyboard::Keyboard;
use crate::reject_result;

#[derive(Clone, Serialize, Deserialize)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    if ans == "✅" {
        cx.answer("Введіть капчу")
            .reply_markup(ReplyMarkup::kb_remove())
            .send()
            .await?;
        let answer = reject_result!(
            cx,
            Captcha::send(&cx).await,
            "Failed to send captcha",
            "Відбулася помилки при створенні капчи",
            Dialogue::Start(state)
        );
        next(Dialogue::ReceiveCaptcha(ReceiveCaptchaState::new(answer)))
    } else {
        cx.answer(
            "Щоб продовжити роботу з ботом, погодьтеся зі збором та обробкою персональних даних у вигляді ПІБ та групи",
        )
        .reply_markup(Keyboard::global().get_agree_keyboard())
        .send()
        .await?;
        next(Dialogue::Start(state))
    }
}
