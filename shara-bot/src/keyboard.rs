use anyhow::Result;
use once_cell::sync::OnceCell;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::database::prize::Prizes;

static INSTANCE: OnceCell<Keyboard> = OnceCell::new();

pub struct Keyboard {
    agree_keyboard: KeyboardMarkup,
}

pub async fn initialize() -> Result<()> {
    let agree_keyboard = KeyboardMarkup::default()
        .append_row(vec![KeyboardButton::new("✅"), KeyboardButton::new("❌")])
        .resize_keyboard(true);
    let queue = Keyboard { agree_keyboard };
    INSTANCE
        .set(queue)
        .map_err(|_| anyhow::anyhow!("Failed to initialize database!"))
}

impl Keyboard {
    pub fn global() -> &'static Keyboard {
        INSTANCE.get().expect("Pool isn't initialized")
    }

    pub fn get_agree_keyboard(&self) -> KeyboardMarkup {
        self.agree_keyboard.clone()
    }

    pub async fn get_prize_keyboard(
        &self,
        prizes: Prizes,
    ) -> Option<InlineKeyboardMarkup> {
        if prizes.0.is_empty() {
            return None;
        }
        let keyboard = prizes.0.iter().enumerate().fold(
            InlineKeyboardMarkup::default(),
            |keyboard, (number, prize)| {
                keyboard.append_row(vec![InlineKeyboardButton::callback(
                    format!("Шара №{}", number + 1),
                    format!("{}", prize.id),
                )])
            },
        );
        Some(keyboard)
    }
}
