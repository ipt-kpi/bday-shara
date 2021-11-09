use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct WaitState;

#[teloxide(subtransition)]
async fn wait(
    state: WaitState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("Очікуйте розіграшу шар!").await?;
    next(Dialogue::Wait(state))
}
