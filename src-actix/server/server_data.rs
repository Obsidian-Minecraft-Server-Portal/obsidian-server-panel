use crate::server::backups::backup_scheduler;
use crate::server::backups::backup_type::BackupType;
use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_properties::ServerProperties;
use crate::server::server_status::ServerStatus;
use crate::server::server_status::ServerStatus::Idle;
use crate::server::server_type::ServerType;
use crate::{app_db, ICON};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_hash::HashIds;
use sqlx::{FromRow, Row, SqlitePool};
use std::path::{Path, PathBuf};

const SERVER_DIRECTORY: &str = "./servers";
#[derive(HashIds, Debug, Clone, FromRow)]
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
    /// Type of backup to perform: 'full', 'incremental', or 'world'
    pub backup_type: BackupType,
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
            backup_type: BackupType::Incremental,
            backup_retention: 7,
            description: None,
            minecraft_version: None,
            server_type: None,
            loader_version: None,
            owner_id: 0,
            created_at: 0,
            updated_at: 0,
            last_started: None,
        }
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
        PathBuf::from(SERVER_DIRECTORY).join(&self.directory)
    }

    /// Update the server structure data
    /// This will not update the database, use `server.save(&SqlitePool)` for that
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
        self.backup_type = server_data.backup_type.clone();
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

    pub async fn get(id: u64, user_id: u64) -> Result<Option<Self>> {
        let pool = app_db::open_pool().await?;
        // All users can view all servers under the new permission system
        let server = Self::get_with_pool(id, &pool).await?;
        pool.close().await;
        Ok(server)
    }

    pub async fn list(user_id: u64) -> Result<Vec<Self>> {
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
        let mut path = PathBuf::from(SERVER_DIRECTORY).join(&dir_name);
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
        let temp_dir = format!("./tmp/{}/mods/", self.id);
        tokio::fs::create_dir_all(&temp_dir).await?;
        let temp_file_path = PathBuf::from(&temp_dir).join(&filename);

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
            .bind(self.id as i64)
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
            .bind(self.id as i64)
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
            sqlx::query("DELETE FROM installed_mods WHERE mod_id = ? AND server_id = ?").bind(mod_id).bind(self.id as i64).execute(&pool).await?;
        }

        pool.close().await;
        Ok(())
    }

    pub async fn initialize_servers(pool: &SqlitePool) -> Result<()> {
        let servers = Self::list_all_with_pool(pool).await?;
        for mut server in servers {
            if let Err(e) = server.start_watch_server_mod_directory_for_changes().await {
                log::error!("Failed to start mod directory watcher for server {}: {}", server.name, e);
            }
            if let Err(e) = server.refresh_installed_mods(pool).await {
                log::error!("Failed to refresh installed mods for server {}: {}", server.name, e);
            }
            if server.auto_start {
                if let Err(e) = server.start_server().await {
                    log::error!("Failed to auto-start server {}: {}", server.name, e);
                }
            }
        }

        // Initialize backup scheduler
        if let Err(e) = backup_scheduler::initialize_backup_scheduler().await {
            log::error!("Failed to initialize backup scheduler: {}", e);
        } else {
            log::info!("Backup scheduler initialized successfully");
        }

        Ok(())
    }
}
