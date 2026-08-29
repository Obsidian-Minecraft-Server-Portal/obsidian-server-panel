use crate::database::{Pool, sql};
use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_data::ServerData;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use sqlx::{Executor, Row as _};

/// Decodes one installed_mods row into `ModData`.
macro_rules! impl_mod_data_from_row {
    ($($row_type:ty),* $(,)?) => {
        $(
            pub(crate) fn mod_data_from_row(row: &$row_type) -> Result<ModData> {
                let authors_str: String = row.try_get("author")?;
                let authors: Vec<String> = authors_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

                let icon_data: Option<String> = row.try_get("icon").ok();
                let icon: Option<Vec<u8>> = icon_data.and_then(|data| general_purpose::STANDARD.decode(data).ok());

                Ok(ModData {
                    mod_id: row.try_get("mod_id")?,
                    name: row.try_get("name")?,
                    version: row.try_get("version")?,
                    description: row.try_get("description")?,
                    authors,
                    icon,
                    modrinth_id: row.try_get("modrinth_id").ok(),
                    curseforge_id: row.try_get("curseforge_id").ok(),
                    filename: row.try_get("filename")?,
                    loader: row.try_get("loader").ok(),
                    path: row.try_get("path").ok(),
                    hash: row.try_get("hash").ok(),
                })
            }
        )*
    };
}

#[cfg(feature = "sqlite")]
impl_mod_data_from_row!(sqlx::sqlite::SqliteRow);
#[cfg(feature = "mysql")]
impl_mod_data_from_row!(sqlx::mysql::MySqlRow);
#[cfg(feature = "postgres")]
impl_mod_data_from_row!(sqlx::postgres::PgRow);

#[cfg(feature = "sqlite")]
static CREATE_INSTALLED_MODS_TABLE_SQL: &str = include_str!("../../../resources/sql/sqlite/installed_mods.sql");
#[cfg(feature = "mysql")]
static CREATE_INSTALLED_MODS_TABLE_SQL: &str = include_str!("../../../resources/sql/mysql/installed_mods.sql");
#[cfg(feature = "postgres")]
static CREATE_INSTALLED_MODS_TABLE_SQL: &str = include_str!("../../../resources/sql/postgres/installed_mods.sql");

pub async fn initialize(pool: &Pool) -> Result<()> {
    pool.execute(CREATE_INSTALLED_MODS_TABLE_SQL).await?;
    for statement in [
        "ALTER TABLE installed_mods ADD COLUMN loader TEXT",
        "ALTER TABLE installed_mods ADD COLUMN path TEXT",
        "ALTER TABLE installed_mods ADD COLUMN hash TEXT",
    ] {
        if let Err(e) = pool.execute(statement).await {
            log::debug!("Skipping installed_mods migration '{}': {}", statement, e);
        }
    }
    backfill_mod_paths(pool).await;
    purge_duplicate_mods(pool).await;
    create_unique_path_index(pool).await;
    Ok(())
}

/// Computes the absolute file path of a mod inside a server's mods directory,
/// mirroring `ServerData::get_directory_path` so stored paths stay canonical.
fn mod_absolute_path(server_directory: &str, filename: &str) -> String {
    let path = crate::server::server_data::get_servers_directory().join(server_directory).join("mods").join(filename);
    match path.canonicalize() {
        Ok(absolute) => absolute.to_string_lossy().into_owned(),
        Err(_) => match std::env::current_dir() {
            Ok(cwd) => cwd.join(path).to_string_lossy().into_owned(),
            Err(_) => path.to_string_lossy().into_owned(),
        },
    }
}

/// Fills in the absolute `path` for rows imported before the column existed.
async fn backfill_mod_paths(pool: &Pool) {
    let rows = match sqlx::query(&*sql(
        r#"SELECT installed_mods.id, installed_mods.filename, servers.directory FROM installed_mods JOIN servers ON installed_mods.server_id = servers.id WHERE installed_mods.path IS NULL"#,
    ))
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            log::warn!("Could not load installed mods for path backfill: {e}");
            return;
        }
    };
    for row in rows {
        let id: i64 = row.try_get("id").unwrap_or(0);
        let filename: String = row.try_get("filename").unwrap_or_default();
        let directory: String = row.try_get("directory").unwrap_or_default();
        if filename.is_empty() {
            continue;
        }
        let path = mod_absolute_path(&directory, &filename);
        if let Err(e) = sqlx::query(&*sql("UPDATE installed_mods SET path = ? WHERE id = ?")).bind(&path).bind(id).execute(pool).await {
            log::warn!("Could not backfill path for installed mod {id}: {e}");
        }
    }
}

/// Removes rows duplicated by path, keeping the oldest; the unique index below
/// must not fail on legacy data. Rows without a backfilled path are left alone.
async fn purge_duplicate_mods(pool: &Pool) {
    // The derived table wrapper keeps MySQL happy ("can't specify target table").
    let statement = "DELETE FROM installed_mods WHERE path IS NOT NULL AND id NOT IN (SELECT * FROM (SELECT MIN(id) FROM installed_mods WHERE path IS NOT NULL GROUP BY path) AS keep)";
    if let Err(e) = pool.execute(statement).await {
        log::warn!("Could not purge duplicate installed mods: {e}");
    }
}

/// Enforces one record per absolute file path across all engines.
async fn create_unique_path_index(pool: &Pool) {
    let statements = [
        #[cfg(any(feature = "sqlite", feature = "postgres"))]
        "CREATE UNIQUE INDEX IF NOT EXISTS uq_installed_mods_path ON installed_mods(path)",
        #[cfg(feature = "mysql")]
        "ALTER TABLE installed_mods ADD UNIQUE KEY uq_installed_mods_path (path)",
    ];
    for statement in statements {
        if let Err(e) = pool.execute(statement).await {
            log::debug!("Skipping installed_mods index '{}': {}", statement, e);
        }
    }
}

impl ServerData {
    pub async fn refresh_installed_mods(&self, pool: &Pool) -> Result<()> {
        // Get current mods from filesystem
        let filesystem_mods: Vec<ModData> = ModData::from_server(self).await?;

        // Get current mods from the database
        let db_mods = self.load_installed_mods(pool).await?;

        // Create collections for efficient lookups (keyed by absolute file path)
        let filesystem_paths: std::collections::HashSet<String> =
            filesystem_mods.iter().filter_map(|mod_data| mod_data.path.clone()).collect();

        let db_paths: std::collections::HashSet<String> = db_mods.iter().filter_map(|mod_data| mod_data.path.clone()).collect();

        // Remove mods that are no longer in the filesystem
        for db_mod in &db_mods {
            let still_on_disk = db_mod.path.as_ref().is_some_and(|path| filesystem_paths.contains(path))
                || filesystem_mods.iter().any(|mod_data| mod_data.filename == db_mod.filename);
            if !still_on_disk {
                self.delete_installed_mod(&db_mod.filename, pool).await?;
            }
        }

        // Add new mods that aren't in the database
        let new_mods: Vec<&ModData> = filesystem_mods
            .iter()
            .filter(|mod_data| match &mod_data.path {
                Some(path) => !db_paths.contains(path),
                None => !db_mods.iter().any(|db_mod| db_mod.filename == mod_data.filename),
            })
            .collect();

        // Insert new mods in batches of 1000
        for batch in new_mods.chunks(1000) {
            let mut tx = pool.begin().await?;

            for mod_data in batch {
                // Encode icon data as a base64 string if present
                let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

                sqlx::query(&*sql(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#))
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

    pub async fn load_installed_mods(&self, pool: &Pool) -> Result<Vec<ModData>> {
        let rows = sqlx::query(&*sql(r#"SELECT * FROM installed_mods WHERE server_id = ?"#)).bind(self.id as i64).fetch_all(pool).await?;

        let mut mods = Vec::new();
        for row in rows {
            mods.push(mod_data_from_row(&row)?);
        }

        Ok(mods)
    }

    pub async fn load_and_save_installed_mods(&self, pool: &Pool) -> Result<()> {
        let mods: Vec<ModData> = ModData::from_server(self).await?;

        // Process mods in batches of 1000
        for batch in mods.chunks(1000) {
            let mut tx = pool.begin().await?;

            for mod_data in batch {
                // Encode icon data as base64 string if present
                let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

                sqlx::query(&*sql(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, loader, path, hash, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#))
					.bind(&mod_data.mod_id)
					.bind(&mod_data.name)
					.bind(mod_data.version.to_string())
					.bind(mod_data.authors.join(","))
					.bind(&mod_data.description)
					.bind(icon_base64)
					.bind(&mod_data.modrinth_id)
					.bind(&mod_data.curseforge_id)
                    .bind(&mod_data.filename)
                    .bind(&mod_data.loader)
                    .bind(&mod_data.path)
                    .bind(&mod_data.hash)
					.bind(self.id as i64)
					.execute(&mut *tx).await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }

    pub async fn insert_installed_mod(&self, mod_data: &ModData, pool: &Pool) -> Result<()> {
        let icon_base64 = mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes));

        sqlx::query(&*sql(r#"insert into installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#))
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

    pub async fn delete_installed_mod(&self, filename: &str, pool: &Pool) -> Result<()> {
        sqlx::query(&*sql(r#"delete from installed_mods where server_id = ? and filename = ?"#)).bind(self.id as i64).bind(filename).execute(pool).await?;

        Ok(())
    }
}
