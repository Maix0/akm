use super::Database;

use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;

super::defineID!(ClientId => "clients");

#[derive(Debug, Clone)]
pub struct TableClients {
    pub id: ClientId,
    pub name: String,
    pub description: String,
}

impl Database {
    pub async fn create_client(
        &self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
    ) -> Result<ClientId> {
        let name = name.as_ref();
        let desc = desc.as_ref();

        let query = sqlx::query!(
            "INSERT INTO clients ('name', 'description') VALUES (?, ?) RETURNING id",
            name,
            desc,
        )
        .fetch_one(&self.inner)
        .await?;
        Ok(ClientId(query.id))
    }

    pub async fn fetch_client(&self, client: ClientId) -> Result<Option<TableClients>> {
        let query = sqlx::query!("SELECT * FROM clients where id = ? LIMIT 1", client.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableClients {
            id: ClientId(s.id),
            name: s.name,
            description: s.description,
        }))
    }

    pub async fn remove_client(&self, client: ClientId) -> Result<bool> {
        sqlx::query!("DELETE FROM clients WHERE id = ?", client.0)
            .execute(&self.inner)
            .await
            .inspect(|s| assert!(s.rows_affected() <= 1, "mutliple clients with the same id"))
            .map(|s| s.rows_affected() == 1)
            .map_err(color_eyre::Report::from)
    }
}
