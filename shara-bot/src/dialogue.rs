use serde::{Deserialize, Serialize};
use teloxide::macros::Transition;

use crate::dialogue::states::{
    BannedState, ReceiveCaptchaState, ReceiveFullNameState, ReceiveGroupState, StartState,
    WaitState,
};

pub mod states;

#[derive(Transition, Serialize, Deserialize)]
pub enum Dialogue {
    Banned(BannedState),
    Start(StartState),
    ReceiveCaptcha(ReceiveCaptchaState),
    ReceiveFullName(ReceiveFullNameState),
    ReceiveGroup(ReceiveGroupState),
    Wait(WaitState),
}

impl Dialogue {
    pub fn is_start(&self) -> bool {
        match &self {
            Dialogue::Start(_) => true,
            _ => false,
        }
    }
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}
