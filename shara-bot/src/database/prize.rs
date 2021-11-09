use std::collections::BTreeMap;
use std::fmt;

#[derive(sqlx::FromRow)]
pub struct Prize {
    pub id: i32,
    pub teacher: String,
    pub subject: String,
    pub prize_type: String,
    pub count: i16,
}

impl fmt::Display for Prize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}) {} [Кількість шар - {}]",
            self.teacher, self.subject, self.prize_type, self.count
        )
    }
}

pub struct Prizes(pub Vec<Prize>);

impl Prizes {
    pub fn get_map(&self) -> BTreeMap<i32, String> {
        self.0
            .iter()
            .map(|prize| (prize.id, format!("{}", prize)))
            .collect()
    }
}
