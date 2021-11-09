use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dialogue::states::ReceiveGroupState;
use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveFullNameState;

static FULL_NAME_REGEX: OnceCell<Regex> = OnceCell::new();

#[teloxide(subtransition)]
async fn receive_full_name(
    state: ReceiveFullNameState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    let regex = FULL_NAME_REGEX.get_or_init(|| {
        Regex::new(r"^(?:[а-щА-ЩЬьЮюЯяЇїІіЄєҐґ']+ ){2}[а-щА-ЩЬьЮюЯяЇїІіЄєҐґ']+$")
            .expect("Failed to create full name regex!")
    });
    if regex.is_match(&ans) {
        let mut full_name = ans.split_whitespace();
        let last_name = full_name.next().unwrap();
        let first_name = full_name.next().unwrap();
        let patronymic = full_name.next().unwrap();
        let receive_group_state = ReceiveGroupState::new(
            last_name.to_string(),
            first_name.to_string(),
            patronymic.to_string(),
        );
        cx.answer("Введіть свою групу у форматі 'ФX-XX', якщо ви магістр то після шифру групи допишіть 'мн' чи 'мп'. (Приклади груп: ФБ-96, ФІ-11мп, ФФ-02мн)").await?;
        next(Dialogue::ReceiveGroup(receive_group_state))
    } else {
        cx.answer("Неправильно введено ПІБ, спробуйте ще раз!")
            .await?;
        next(Dialogue::ReceiveFullName(state))
    }
}
