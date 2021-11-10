use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::database::user::Student;
use crate::database::Database;
use crate::dialogue::states::{ReceiveFullNameState, WaitState};
use crate::dialogue::Dialogue;
use crate::keyboard::Keyboard;
use crate::{reject_option, reject_result};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveGroupState {
    last_name: String,
    first_name: String,
    patronymic: String,
}

impl ReceiveGroupState {
    pub fn new(last_name: String, first_name: String, patronymic: String) -> Self {
        ReceiveGroupState {
            last_name,
            first_name,
            patronymic,
        }
    }
}

static GROUP_REGEX: OnceCell<Regex> = OnceCell::new();

#[teloxide(subtransition)]
async fn receive_group(
    state: ReceiveGroupState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    let regex = GROUP_REGEX.get_or_init(|| {
        Regex::new(r"Ф(Б|Е|І|Ф)-\d\d(мп|мн)?+$").expect("Failed to create phone number regex!")
    });
    if regex.is_match(&ans) {
        let database = Database::global();
        let group_code = reject_result!(
            cx,
            database.get_group_code(ans.clone()).await,
            "Database error while register",
            "Помилка при перевірки групи!",
            Dialogue::ReceiveGroup(state)
        );
        let group_code = reject_option!(
            cx,
            group_code,
            "Вказану групу не вдалося знайти у списку шар, спробуйте ще раз!",
            Dialogue::ReceiveGroup(state)
        );
        let user = reject_option!(
            cx,
            cx.update.from(),
            "Не вдалося отримати дані про користувача, спробуйте ще раз ввести ПІБ",
            Dialogue::ReceiveFullName(ReceiveFullNameState)
        );
        let student = Student {
            chat_id: cx.update.chat.id,
            username: user.username.as_ref().map_or(String::new(), String::from),
            last_name: state.last_name.clone(),
            first_name: state.first_name.clone(),
            patronymic: state.patronymic.clone(),
            group_code,
        };
        let _ = reject_result!(
            cx,
            database.register(student).await,
            "Database error while register",
            "Помилка при реєстрації студента!",
            Dialogue::ReceiveFullName(ReceiveFullNameState)
        );
        cx.answer(format!(
            "<b>Підсумкові дані:</b> \n\
            Прізвище: {} \n\
            Ім'я: {} \n\
            По батькові: {} \n\
            Група: {}",
            state.last_name, state.first_name, state.patronymic, ans,
        ))
        .parse_mode(ParseMode::Html)
        .await?;
        let prizes = reject_result!(
            cx,
            database.get_prizes(cx.chat_id()).await,
            "Database error while get prizes",
            "Помилка при отриманні шар",
            Dialogue::ReceiveGroup(state)
        );
        let mut message = cx.answer(format!("{}", prizes))
            .parse_mode(ParseMode::Html);
        if let Some(keyboard) = Keyboard::global().get_prize_keyboard(prizes).await {
            message = message.reply_markup(keyboard)
        }
        message.await?;
        next(Dialogue::Wait(WaitState))
    } else {
        cx.answer("Неправильно введено групу, спробуйте ще раз!")
            .await?;
        next(Dialogue::ReceiveGroup(state))
    }
}
