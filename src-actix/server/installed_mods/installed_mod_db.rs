use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_data::ServerData;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use sqlx::{Executor, Row, SqlitePool};

pub async fn initialize(pool: &SqlitePool) -> Result<()> {
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS installed_mods
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id        TEXT NOT NULL,
    name          TEXT NOT NULL,
    version       TEXT NOT NULL,
    author        TEXT NOT NULL,
    description   TEXT NOT NULL,
    icon          TEXT DEFAULT NULL,
    modrinth_id   TEXT DEFAULT NULL,
    curseforge_id TEXT DEFAULT NULL,
    filename      TEXT DEFAULT NULL,
    server_id     INTEGER REFERENCES servers (id) ON DELETE CASCADE
)
		
		"#,
    )
    .await?;

    Ok(())
}

impl ServerData {
    pub async fn refresh_installed_mods(&self, pool: &SqlitePool) -> Result<()> {
        // Get current mods from filesystem
        let filesystem_mods: Vec<ModData> = ModData::from_server(self).await?;

        // Get current mods from the database
        let db_mods = self.load_installed_mods(pool).await?;

        // Create collections for efficient lookups
        let filesystem_filenames: std::collections::HashSet<String> =
            filesystem_mods.iter().filter_map(|mod_data| Option::from(mod_data.filename.clone())).collect();

        let db_filenames: std::collections::HashSet<String> = db_mods.iter().filter_map(|mod_data| Option::from(mod_data.filename.clone())).collect();

        // Remove mods that are no longer in the filesystem
        for db_mod in &db_mods {
            if !filesystem_filenames.contains(&db_mod.filename) {
                self.delete_installed_mod(&db_mod.filename, pool).await?;
            }
        }

        // Add new mods that aren't in the database
        let new_mods: Vec<&ModData> = filesystem_mods.iter().filter(|mod_data| !db_filenames.contains(&mod_data.filename)).collect();

        // Insert new mods in batches of 1000
        for batch in new_mods.chunks(1000) {
            let mut tx = pool.begin().await?;

            for mod_data in batch {
                // Encode icon data as a base64 string if present
                let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

                sqlx::query(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
                    .bind(&mod_data.mod_id)
                    .bind(&mod_data.name)
                    .bind(mod_data.version.to_string())
                    .bind(mod_data.authors.join(","))
                    .bind(&mod_data.description)
                    .bind(icon_base64)
                    .bind(&mod_data.modrinth_id)
                    .bind(&mod_data.curseforge_id)
                    .bind(&mod_data.filename)
                    .bind(self.id as i64)
                    .execute(&mut *tx).await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }

    pub async fn load_installed_mods(&self, pool: &SqlitePool) -> Result<Vec<ModData>> {
        let rows = sqlx::query(r#"SELECT * FROM installed_mods WHERE server_id = ?"#).bind(self.id as i64).fetch_all(pool).await?;

        let mut mods = Vec::new();
        for row in rows {
            let authors_str: String = row.get("author");
            let authors: Vec<String> = authors_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

            let icon_data: Option<String> = row.get("icon");
            let icon: Option<Vec<u8>> = icon_data.and_then(|data| general_purpose::STANDARD.decode(data).ok());

            let mod_data = ModData {
                mod_id: row.get("mod_id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                authors,
                icon,
                modrinth_id: row.get("modrinth_id"),
                curseforge_id: row.get("curseforge_id"),
                filename: row.get("filename"),
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
                let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

                sqlx::query(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
					.bind(&mod_data.mod_id)
					.bind(&mod_data.name)
					.bind(mod_data.version.to_string())
					.bind(mod_data.authors.join(","))
					.bind(&mod_data.description)
					.bind(icon_base64)
					.bind(&mod_data.modrinth_id)
					.bind(&mod_data.curseforge_id)
                    .bind(&mod_data.filename)
					.bind(self.id as i64)
					.execute(&mut *tx).await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }

    pub async fn insert_installed_mod(&self, mod_data: &ModData, pool: &SqlitePool) -> Result<()> {
        let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

        sqlx::query(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
            .bind(&mod_data.mod_id)
            .bind(&mod_data.name)
            .bind(mod_data.version.to_string())
            .bind(mod_data.authors.join(","))
            .bind(&mod_data.description)
            .bind(icon_base64)
            .bind(&mod_data.modrinth_id)
            .bind(&mod_data.curseforge_id)
            .bind(&mod_data.filename)
            .bind(self.id as i64)
            .execute(pool).await?;
        Ok(())
    }

    pub async fn delete_installed_mod(&self, filename: &str, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"delete from installed_mods where server_id = ? and filename = ?"#).bind(self.id as i64).bind(filename).execute(pool).await?;

        Ok(())
    }

    pub async fn clear_saved_installed_mods(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"delete from installed_mods where server_id = ?"#).bind(self.id as i64).execute(pool).await?;

        Ok(())
    }
}
