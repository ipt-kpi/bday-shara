use std::collections::BTreeMap;
use std::fmt;

#[derive(sqlx::FromRow)]
pub struct Prize {
    pub id: i32,
    pub teacher: String,
    pub subject: String,
    pub prize_type: String,
    pub count: i16,
    #[sqlx(default)]
    pub groups: Option<String>,
    pub selected: bool
}

impl fmt::Display for Prize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.groups {
            Some(groups) => {
                write!(
                    f,
                    "{}\n\
                    <b>Предмет:</b> {}\n\
                    <b>{}</b> [Кількість шар - {}]\n\
                    <b>Розігрується серед груп:</b> [{}]",
                    self.teacher, self.subject, self.prize_type, self.count, groups
                )
            },
            None => {
                write!(
                    f,
                    "{}\n\
                    <b>Предмет</b>: {}\n\
                    <b>{}</b> [Кількість шар - {}]\n\
                    <b>Розігрується серед вашої групи</b>",
                    self.teacher, self.subject, self.prize_type, self.count
                )
            }
        }
    }
}

pub struct Prizes(pub Vec<Prize>);

impl fmt::Display for Prizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "Не вдалося знайти шари на обрану групу")
        } else {
            self.0.iter().enumerate().fold(Ok(()), |result, (number, prize)| {
                let selection = if prize.selected {
                    "✅"
                } else {
                    "❌"
                };
                result.and_then(|_| writeln!(f, "{} Шара №{}: {}", selection, number + 1, prize))
            })
        }
    }
}

impl Prizes {
    pub fn get_map(&self) -> BTreeMap<i32, String> {
        self.0
            .iter()
            .map(|prize| (prize.id, format!("{}", prize)))
            .collect()
    }
}
