use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;

pub mod clientkeys;
pub mod clients;
pub mod keys;
pub mod users;

#[derive(Clone, Debug)]
pub struct Database {
    inner: sqlx::SqlitePool,
}

pub type DateTime = chrono::DateTime<chrono::Utc>;

impl Database {
    const INIT_SCRIPT: &str = include_str!("./database/init.sql");

    pub async fn new(path: impl AsRef<str>) -> Result<Self> {
        let path = path.as_ref();
        let db = sqlx::SqlitePool::connect(path).await?;
        {
            let mut s = db.execute_many(Self::INIT_SCRIPT);
            while s.next().await.transpose()?.is_some() {}
        }
        Ok(Database { inner: db })
    }
}

macro_rules! defineID {
    ($($name:ident => $table:literal),*$(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, ::serde::Serialize)]
            pub struct $name(pub(super) i64);
            impl $name {
                pub fn inner(self) -> i64 { self.0 }
                pub async fn from_raw(database: &$crate::database::Database, raw: i64) -> ::color_eyre::Result<Option<Self>> {
                    let res = ::sqlx::query(concat!("SELECT id FROM ", $table, " WHERE id = ? LIMIT 1")).bind(raw).fetch_optional(&database.inner).await?;

                Ok(res.map(|s| ::sqlx::Row::get::<i64, _>(&s, 0)).map($name))
                }
            }
        )*
    };
}
pub(crate) use defineID;
