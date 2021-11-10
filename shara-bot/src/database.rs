use anyhow::Result;
use futures::future::BoxFuture;
use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;
use serde::Serialize;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Row};

use std::fmt::{Debug, Display};
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{Serializer, Storage};

use crate::database::prize::{Prize, Prizes};
use crate::database::user::Student;

pub mod prize;
pub mod user;

static INSTANCE: OnceCell<Database<Json>> = OnceCell::new();

pub struct Database<S> {
    pool: PgPool,
    serializer: S,
}

pub async fn initialize(max_connections: u32, url: &str) -> Result<()> {
    INSTANCE
        .set(Database {
            pool: PgPoolOptions::new()
                .max_connections(max_connections)
                .connect(url)
                .await?,
            serializer: Json,
        })
        .map_err(|_| anyhow::anyhow!("Failed to initialize database!"))
}

impl Database<Json> {
    pub fn global() -> &'static Database<Json> {
        INSTANCE.get().expect("Pool isn't initialized")
    }

    pub async fn register(&self, student: Student) -> Result<()> {
        sqlx::query("INSERT INTO student(chat_id, username, last_name, first_name, patronymic, group_code) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(student.chat_id)
            .bind(student.username)
            .bind(student.last_name)
            .bind(student.first_name)
            .bind(student.patronymic)
            .bind(student.group_code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_group_code(&self, group_code: String) -> Result<Option<i32>> {
        sqlx::query("SELECT id FROM group_code WHERE name = $1")
            .bind(group_code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|optional_row| optional_row.map(|row| row.get(0)))
    }

    pub async fn get_prizes(&self, chat_id: i64) -> Result<Prizes> {
        sqlx::query_as::<_, Prize>(r#"SELECT * FROM get_prizes($1)"#)
            .bind(chat_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|prize| Prizes(prize))
    }

    pub async fn mark_prize(&self, chat_id: i64, prize: i32) -> Result<()> {
        sqlx::query("SELECT * FROM mark_prize($1, $2)")
            .bind(chat_id)
            .bind(prize)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_banned(&self, id: i64) -> Result<bool> {
        sqlx::query("SELECT banned FROM student WHERE chat_id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|optional_row| optional_row.map(|row| row.get(0)).unwrap_or(false))
    }

    pub async fn refresh_user_state(&self, id: i64) -> Result<()> {
        let _id: Option<i32> = sqlx::query("SELECT id FROM student WHERE chat_id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map(|id| id.map(|row| row.get(0)))?;
        Ok(())
    }
}

impl<S, D> Storage<D> for &'static Database<S>
where
    S: Send + Sync + Serializer<D> + 'static,
    D: Send + Serialize + DeserializeOwned + 'static,
    <S as Serializer<D>>::Error: Debug + Display,
{
    type Error = anyhow::Error;

    fn remove_dialogue(
        self: Arc<Self>,
        chat_id: i64,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            Ok(match get_dialogue(&self.pool, chat_id).await? {
                Some(d) => {
                    let prev_dialogue = self.serializer.deserialize(&d).map_err(|error| {
                        anyhow::anyhow!("dialogue serialization error: {}", error)
                    })?;
                    sqlx::query("DELETE FROM teloxide_dialogues WHERE chat_id = $1")
                        .bind(chat_id)
                        .execute(&self.pool)
                        .await?;
                    Some(prev_dialogue)
                }
                _ => None,
            })
        })
    }

    fn update_dialogue(
        self: Arc<Self>,
        chat_id: i64,
        dialogue: D,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            let prev_dialogue = get_dialogue(&self.pool, chat_id)
                .await?
                .map(|d| {
                    self.serializer
                        .deserialize(&d)
                        .map_err(|error| anyhow::anyhow!("Database deserialize error: {}", error))
                })
                .transpose()?;
            let upd_dialogue = self
                .serializer
                .serialize(&dialogue)
                .map_err(|error| anyhow::anyhow!("Database serialize error: {}", error))?;
            self.pool
                .acquire()
                .await?
                .execute(
                    sqlx::query(
                        r#"
            INSERT INTO teloxide_dialogues VALUES ($1, $2)
            ON CONFLICT(chat_id) DO UPDATE SET dialogue=excluded.dialogue
                                "#,
                    )
                    .bind(chat_id)
                    .bind(upd_dialogue),
                )
                .await
                .unwrap();
            Ok(prev_dialogue)
        })
    }
}

async fn get_dialogue(pool: &PgPool, chat_id: i64) -> Result<Option<Box<Vec<u8>>>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct DialogueDbRow {
        dialogue: Vec<u8>,
    }

    Ok(sqlx::query_as::<_, DialogueDbRow>(
        "SELECT dialogue FROM teloxide_dialogues WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?
    .map(|r| Box::new(r.dialogue)))
}
