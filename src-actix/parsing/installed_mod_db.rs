use crate::parsing::mod_data::ModData;
use crate::server::server_data::ServerData;
use anyhow::Result;
use sqlx::{Executor, SqlitePool, Row};
use base64::{Engine as _, engine::general_purpose};

pub async fn initialize(pool: &SqlitePool) -> Result<()> {
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS installed_mods
(
    id            INTEGER auto_increment PRIMARY KEY,
    mod_id        TEXT NOT NULL,
    name          TEXT NOT NULL,
    version       TEXT NOT NULL,
    author        TEXT NOT NULL,
    description   TEXT NOT NULL,
    icon          TEXT DEFAULT NULL,
    modrinth_id   TEXT DEFAULT NULL,
    curseforge_id TEXT DEFAULT NULL,
    server_id INTEGER REFERENCES servers(id) ON DELETE CASCADE
)
		
		"#,
    )
    .await?;

    Ok(())
}

impl ServerData {
    pub async fn refresh_installed_mods(&self, pool: &SqlitePool) -> Result<()> {
        self.clear_saved_installed_mods(pool).await?;
        self.load_and_save_installed_mods(pool).await
    }

    pub async fn load_installed_mods(&self, pool: &SqlitePool) -> Result<Vec<ModData>> {
        let rows = sqlx::query(
            r#"SELECT mod_id, name, version, author, description, icon, modrinth_id, curseforge_id
               FROM installed_mods
               WHERE server_id = ?"#
        )
        .bind(self.id as i64)
        .fetch_all(pool)
        .await?;

        let mut mods = Vec::new();
        for row in rows {
            let authors_str: String = row.get("author");
            let authors: Vec<String> = authors_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let icon_data: Option<String> = row.get("icon");
            let icon: Option<Vec<u8>> = icon_data
                .and_then(|data| general_purpose::STANDARD.decode(data).ok());

            let mod_data = ModData {
                mod_id: row.get("mod_id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                authors,
                icon,
                modrinth_id: row.get("modrinth_id"),
                curseforge_id: row.get("curseforge_id"),
            };
            mods.push(mod_data);
        }

        Ok(mods)
    }

    pub async fn load_and_save_installed_mods(&self, pool: &SqlitePool) -> Result<()> {
        let mods: Vec<ModData> = ModData::from_server(self).await?;

        // Process mods in batches of 1000
        for batch in mods.chunks(1000) {
            let mut tx = pool.begin().await?;

            for mod_data in batch {
                // Encode icon data as base64 string if present
                let icon_base64 = mod_data.icon.as_ref()
                    .map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

                sqlx::query(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
					.bind(&mod_data.mod_id)
					.bind(&mod_data.name)
					.bind(mod_data.version.to_string())
					.bind(mod_data.authors.join(","))
					.bind(&mod_data.description)
					.bind(icon_base64)
					.bind(&mod_data.modrinth_id)
					.bind(&mod_data.curseforge_id)
					.bind(self.id as i64)
					.execute(&mut *tx).await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }
    pub async fn clear_saved_installed_mods(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"delete from installed_mods where server_id = ?"#).bind(self.id as i64).execute(pool).await?;

        Ok(())
    }
}
