use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_properties::ServerProperties;
use crate::server::server_status::ServerStatus;
use crate::server::server_status::ServerStatus::Idle;
use crate::server::server_type::ServerType;
use crate::{app_db, ICON};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_hash::HashIds;
use sqlx::{FromRow, Row, MySqlPool, Error};
use sqlx::mysql::MySqlRow;
use std::path::{PathBuf};

/// Get the servers directory from settings, with fallback to default
fn get_servers_directory() -> PathBuf {
    if let Ok(settings) = crate::settings::load_settings() {
        settings.storage.servers_directory
    } else {
        PathBuf::from("./meta/servers")
    }
}

/// Get the temp directory from settings, with fallback to default
fn get_temp_directory() -> PathBuf {
    if let Ok(settings) = crate::settings::load_settings() {
        settings.storage.temp_directory
    } else {
        PathBuf::from("./meta/temp")
    }
}
#[derive(HashIds, Debug, Clone)]
pub struct ServerData {
    /// Unique identifier for the server
    #[hash]
    pub id: u64,
    /// Name of the server, e.g. 'My Minecraft Server'
    pub name: String,
    /// Directory name where server files are stored, e.g. 'my_minecraft_server'
    pub directory: String,
    /// Path to Java executable, e.g. '/usr/bin/java' or 'java' for system PATH
    pub java_executable: String,
    /// Additional JVM arguments excluding -Xmx and -Xms
    pub java_args: String,
    /// Maximum memory in GB for JVM -Xmx argument
    pub max_memory: u8,
    /// Minimum memory in GB for JVM -Xms argument
    pub min_memory: u8,
    /// Additional Minecraft server arguments
    pub minecraft_args: String,
    /// Name/path of the server JAR file
    pub server_jar: String,
    /// Whether UPnP port forwarding is enabled
    pub upnp: bool,
    /// Server status: 'stopped', 'starting', 'running', 'stopping', 'error'
    pub status: ServerStatus,
    /// Whether the server should start automatically on boot
    pub auto_start: bool,
    /// Whether the server should restart automatically if it crashes
    pub auto_restart: bool,
    /// Whether automatic backups are enabled
    pub backup_enabled: bool,
    /// Backup interval in minutes
    pub backup_cron: String,
    /// Number of backups to keep for retention
    pub backup_retention: u32,
    /// Optional server description
    pub description: Option<String>,
    /// Minecraft version, e.g. '1.20.1', '1.19.4', or 'custom'
    pub minecraft_version: Option<String>,
    /// Server type: 'vanilla', 'fabric', 'forge', 'neoforge', 'quilt', or 'custom'
    pub server_type: Option<ServerType>,
    /// Loader version e.g. '0.14.0', '1.20.1-44.1.23', or 'custom'
    pub loader_version: Option<String>,
    /// ID of the user who owns this server
    #[hash]
    pub owner_id: u64,
    /// Timestamp of when the server was created (seconds since epoch)
    pub created_at: u64,
    /// Timestamp of when the server was last updated (seconds since epoch)
    pub updated_at: u64,
    /// Timestamp of when the server was last started (seconds since epoch)
    pub last_started: Option<u64>,
    /// Timestamp of when the server was last checked for updates (seconds since epoch)
    pub last_update_check: Option<u64>,
    /// Whether an update is available for this server
    pub update_available: bool,
    /// The latest version available for this server (loader or minecraft version)
    pub latest_version: Option<String>,
}

impl Default for ServerData {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            directory: String::new(),
            java_executable: String::new(),
            java_args: String::new(),
            max_memory: 2,
            min_memory: 1,
            minecraft_args: String::new(),
            server_jar: "".to_string(),
            upnp: false,
            status: Idle,
            auto_start: false,
            auto_restart: false,
            backup_enabled: false,
            backup_cron: "0 0 * * * *".to_string(),
            backup_retention: 7,
            description: None,
            minecraft_version: None,
            server_type: None,
            loader_version: None,
            owner_id: 0,
            created_at: 0,
            updated_at: 0,
            last_started: None,
            last_update_check: None,
            update_available: false,
            latest_version: None,
        }
    }
}

impl<'a> FromRow<'a, MySqlRow> for ServerData {
    fn from_row(row: &'a MySqlRow) -> Result<Self, Error> {
        // MySQL INT returns as i32, convert to u64
        let id: u64 = row.try_get::<u32, _>("id")? as u64;
        let name: String = row.try_get("name")?;
        let directory: String = row.try_get("directory")?;
        let java_executable: String = row.try_get("java_executable")?;
        let java_args: String = row.try_get("java_args")?;
        let max_memory: i8 = row.try_get("max_memory")?;
        let min_memory: i8 = row.try_get("min_memory")?;
        let minecraft_args: String = row.try_get("minecraft_args")?;
        let server_jar: String = row.try_get("server_jar")?;
        let upnp: i8 = row.try_get("upnp")?;
        let status: i8 = row.try_get("status")?;
        let auto_start: i8 = row.try_get("auto_start")?;
        let auto_restart: i8 = row.try_get("auto_restart")?;
        let backup_enabled: i8 = row.try_get("backup_enabled")?;
        let backup_cron: String = row.try_get("backup_cron")?;
        let backup_retention: i32 = row.try_get("backup_retention")?;
        let description: Option<String> = row.try_get("description")?;
        let minecraft_version: Option<String> = row.try_get("minecraft_version")?;
        let server_type: Option<i8> = row.try_get("server_type")?;
        let loader_version: Option<String> = row.try_get("loader_version")?;
        let owner_id: u64 = row.try_get::<u32, _>("owner_id")? as u64;
        let created_at: i32 = row.try_get("created_at")?;
        let updated_at: i32 = row.try_get("updated_at")?;
        let last_started: Option<i32> = row.try_get("last_started")?;
        let last_update_check: Option<i32> = row.try_get("last_update_check")?;
        let update_available: i8 = row.try_get("update_available")?;
        let latest_version: Option<String> = row.try_get("latest_version")?;

        Ok(ServerData {
            id,
            name,
            directory,
            java_executable,
            java_args,
            max_memory: max_memory as u8,
            min_memory: min_memory as u8,
            minecraft_args,
            server_jar,
            upnp: upnp != 0,
            status: ServerStatus::from(status as u8),
            auto_start: auto_start != 0,
            auto_restart: auto_restart != 0,
            backup_enabled: backup_enabled != 0,
            backup_cron,
            backup_retention: backup_retention as u32,
            description,
            minecraft_version,
            server_type: server_type.map(|t| ServerType::from(t as u8)),
            loader_version,
            owner_id,
            created_at: created_at as u64,
            updated_at: updated_at as u64,
            last_started: last_started.map(|t| t as u64),
            last_update_check: last_update_check.map(|t| t as u64),
            update_available: update_available != 0,
            latest_version,
        })
    }
}

impl ServerData {
    pub fn new(
        name: String,
        server_type: ServerType,
        minecraft_version: String,
        loader_version: Option<String>,
        java_executable: String,
        owner_id: u64,
    ) -> Self {
        Self {
            name: name.clone(),
            directory: Self::generate_directory_name(name.as_str()),
            minecraft_version: Some(minecraft_version),
            server_type: Some(server_type),
            loader_version,
            java_executable,
            owner_id,
            ..Self::default()
        }
    }

    pub fn get_directory_path(&self) -> PathBuf {
        let path = get_servers_directory().join(&self.directory);
        // Convert to absolute path to avoid issues with relative paths
        match path.canonicalize() {
            Ok(absolute_path) => absolute_path,
            Err(_) => {
                // If canonicalize fails (e.g., path doesn't exist yet), 
                // make it absolute by prepending current working directory
                match std::env::current_dir() {
                    Ok(cwd) => cwd.join(path),
                    Err(_) => path, // Fallback to original path if all else fails
                }
            }
        }
    }

    /// Update the server structure data
    /// This will not update the database, use `server.save(&MySqlPool)` for that
    pub fn update(&mut self, server_data: &ServerData) -> Result<()> {
        self.name = server_data.name.clone();
        self.directory = server_data.directory.clone();
        self.java_executable = server_data.java_executable.clone();
        self.java_args = server_data.java_args.clone();
        self.max_memory = server_data.max_memory;
        self.min_memory = server_data.min_memory;
        self.minecraft_args = server_data.minecraft_args.clone();
        self.server_jar = server_data.server_jar.clone();
        self.upnp = server_data.upnp;
        self.status = server_data.status.clone();
        self.auto_start = server_data.auto_start;
        self.auto_restart = server_data.auto_restart;
        self.backup_enabled = server_data.backup_enabled;
        self.backup_cron = server_data.backup_cron.clone();
        self.backup_retention = server_data.backup_retention;
        self.description = server_data.description.clone();
        self.minecraft_version = server_data.minecraft_version.clone();
        self.server_type = server_data.server_type.clone();
        self.loader_version = server_data.loader_version.clone();
        self.owner_id = server_data.owner_id;
        self.last_started = server_data.last_started;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    pub async fn get(id: u64, _user_id: u64) -> Result<Option<Self>> {
        let pool = app_db::open_pool().await?;
        // All users can view all servers under the new permission system
        let server = Self::get_with_pool(id, &pool).await?;
        pool.close().await;
        Ok(server)
    }

    pub async fn list(_user_id: u64) -> Result<Vec<Self>> {
        let pool = app_db::open_pool().await?;
        // All users can see all servers under the new permission system
        let servers = Self::list_all_with_pool(&pool).await?;
        pool.close().await;
        Ok(servers)
    }

    pub async fn save(&self) -> Result<()> {
        let pool = app_db::open_pool().await?;
        self.save_with_pool(&pool).await?;
        pool.close().await;
        Ok(())
    }

    pub fn get_icon(&self) -> Vec<u8> {
        let icon_path = self.get_directory_path().join("server-icon.png");
        if icon_path.exists() {
            let result = std::fs::read(icon_path);
            if let Ok(data) = result {
                return data;
            } else if let Err(e) = result {
                log::error!("Failed to read server icon: {}", e);
            }
        }
        ICON.to_vec()
    }

    pub fn get_server_properties(&self) -> Result<ServerProperties> {
        let properties_path = self.get_directory_path().join("server.properties");
        ServerProperties::load(properties_path)
    }

    fn generate_directory_name(name: &str) -> String {
        let dir_name = regex::Regex::new(r"[^a-zA-Z0-9_\-]").unwrap().replace_all(name, "_").to_string().to_lowercase();
        let mut path = get_servers_directory().join(&dir_name);
        let mut index = 1u32;
        loop {
            if !path.exists() {
                return if let Some(filename) = path.file_name() { filename.to_string_lossy().to_string() } else { dir_name };
            }
            path.set_file_name(format!("{} ({})", dir_name, index)); // e.g. "my_minecraft_server (1)"
            index += 1;
        }
    }

    pub async fn get_installed_mods(&self) -> Result<Vec<ModData>> {
        let pool = app_db::open_pool().await?;
        let saved = self.load_installed_mods(&pool).await?;
        pool.close().await;

        if !saved.is_empty() {
            Ok(saved)
        } else {
            // If no mods in database, scan filesystem and save to database
            let mods = ModData::from_server(self).await?;
            if !mods.is_empty() {
                let pool = app_db::open_pool().await?;
                let _ = self.load_and_save_installed_mods(&pool).await;
                pool.close().await;
            }
            Ok(mods)
        }
    }

    pub async fn sync_installed_mods(&self) -> Result<()> {
        let pool = app_db::open_pool().await?;
        let result = self.refresh_installed_mods(&pool).await;
        pool.close().await;
        result
    }

    pub async fn download_and_install_mod(
        &self,
        download_url: &str,
        filename: String,
        version: Option<String>,
        modrinth_id: Option<String>,
        curseforge_id: Option<String>,
        icon: Option<String>,
    ) -> Result<ModData> {
        // Create mods directory if it doesn't exist
        let mods_dir = self.get_directory_path().join("mods");
        std::fs::create_dir_all(&mods_dir)?;

        // Download the mod file
        let response = reqwest::get(download_url).await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download mod: HTTP {}", response.status()));
        }

        // Ensure .jar extension
        let filename = if filename.ends_with(".jar") { filename } else { format!("{}.jar", filename) };
        let temp_dir = get_temp_directory().join(format!("{}/mods/", self.id));
        tokio::fs::create_dir_all(&temp_dir).await?;
        let temp_file_path = temp_dir.join(&filename);

        // Write the file after inserting into the database to prevent against multiple inserts from the file watcher
        let bytes = response.bytes().await?;
        tokio::fs::write(&temp_file_path, bytes).await?;

        // Parse mod data from the downloaded file
        let mod_data = ModData::from_path(&temp_file_path).await?.ok_or_else(|| anyhow::anyhow!("Failed to parse mod data from downloaded file"))?;

        // Save to database
        let pool = app_db::open_pool().await?;
        let icon_base64 =
            if let Some(icon) = icon { &Some(icon) } else { &mod_data.icon.as_ref().map(|icon_bytes| general_purpose::STANDARD.encode(icon_bytes)) };
        let modrinth_id = if let Some(id) = modrinth_id { &Some(id) } else { &mod_data.modrinth_id };
        let curseforge_id = if let Some(id) = curseforge_id { &Some(id) } else { &mod_data.curseforge_id };

        sqlx::query(r#"INSERT INTO installed_mods (mod_id, name, version, author, description, icon, modrinth_id, curseforge_id, filename, server_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
            .bind(&mod_data.mod_id)
            .bind(&mod_data.name)
            .bind(version.unwrap_or(mod_data.version.clone()))
            .bind(mod_data.authors.join(","))
            .bind(&mod_data.description)
            .bind(icon_base64)
            .bind(modrinth_id)
            .bind(curseforge_id)
            .bind(&filename)
            .bind(self.id as u32)
            .execute(&pool)
            .await?;

        pool.close().await;

        tokio::fs::rename(temp_file_path, mods_dir.join(&filename)).await?;
        Ok(mod_data)
    }

    pub async fn delete_mod(&self, mod_id: &str) -> Result<()> {
        let pool = app_db::open_pool().await?;

        // Get the filename from database
        let row = sqlx::query("SELECT filename FROM installed_mods WHERE mod_id = ? AND server_id = ?")
            .bind(mod_id)
            .bind(self.id as u32)
            .fetch_optional(&pool)
            .await?;

        if let Some(row) = row {
            let filename: Option<String> = row.get("filename");

            // Delete from filesystem if filename is available
            if let Some(filename) = filename {
                let file_path = self.get_directory_path().join("mods").join(filename);
                if file_path.exists() {
                    tokio::fs::remove_file(file_path).await?;
                }
            }

            // Delete from database
            sqlx::query("DELETE FROM installed_mods WHERE mod_id = ? AND server_id = ?").bind(mod_id).bind(self.id as u32).execute(&pool).await?;
        }

        pool.close().await;
        Ok(())
    }

    pub async fn initialize_servers(pool: &MySqlPool) -> Result<()> {
        let servers = Self::list_all_with_pool(pool).await?;
        for mut server in servers {
            if let Err(e) = server.start_watch_server_mod_directory_for_changes().await {
                log::error!("Failed to start mod directory watcher for server {}: {}", server.name, e);
            }
            if let Err(e) = server.refresh_installed_mods(pool).await {
                log::error!("Failed to refresh installed mods for server {}: {}", server.name, e);
            }
            if server.auto_start
                && let Err(e) = server.start_server().await {
                    log::error!("Failed to auto-start server {}: {}", server.name, e);
                }
        }

        Ok(())
    }
}
