use actix_web::dev::Path;
use anyhow::Result;
use serde_hash::HashIds;
use sqlx::FromRow;
use std::path::PathBuf;

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
    pub java_executable: Option<String>,
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
    pub status: String,
    /// Whether server should start automatically on boot
    pub auto_start: bool,
    /// Whether server should restart automatically if it crashes
    pub auto_restart: bool,
    /// Whether automatic backups are enabled
    pub backup_enabled: bool,
    /// Backup interval in minutes
    pub backup_interval: u64,
    /// Optional server description
    pub description: Option<String>,
    /// Minecraft version, e.g. '1.20.1', '1.19.4', or 'custom'
    pub minecraft_version: Option<String>,
    /// Server type: 'vanilla', 'fabric', 'forge', 'neoforge', 'quilt', or 'custom'
    pub server_type: Option<String>,
    /// Loader version e.g. '0.14.0', '1.20.1-44.1.23', or 'custom'
    pub loader_version: Option<String>,
    /// ID of the user who owns this server
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
            java_executable: None,
            java_args: String::new(),
            max_memory: 2,
            min_memory: 1,
            minecraft_args: String::new(),
            server_jar: "".to_string(),
            upnp: false,
            status: "stopped".to_string(),
            auto_start: false,
            auto_restart: false,
            backup_enabled: false,
            backup_interval: 60,
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
    pub fn new(name: String, server_type: String, minecraft_version: String, loader_version: Option<String>, owner_id: u64) -> Self {
        Self {
            name: name.clone(),
            directory: Self::generate_directory_name(name.as_str()),
            minecraft_version: Some(minecraft_version),
            server_type: Some(server_type),
            loader_version,
            owner_id,
            ..Self::default()
        }
    }

    pub fn get_start_command(&self) -> String {
        let java_executable = if let Some(java_executable) = &self.java_executable { java_executable.clone() } else { "java".to_string() };
        let mut command = format!("{} -Xmx{}G -Xms{}G {}", java_executable, self.max_memory, self.min_memory, self.java_args);

        if !self.minecraft_args.is_empty() {
            command.push_str(&format!(" {}", self.minecraft_args));
        }

        command.push_str(&format!(" -jar {} nogui", self.server_jar));
        command
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
        self.backup_interval = server_data.backup_interval;
        self.description = server_data.description.clone();
        self.minecraft_version = server_data.minecraft_version.clone();
        self.server_type = server_data.server_type.clone();
        self.loader_version = server_data.loader_version.clone();
        self.owner_id = server_data.owner_id;
        self.last_started = server_data.last_started;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    fn generate_directory_name(name: &str) -> String {
        let dir_name = regex::Regex::new(r"[^a-zA-Z0-9_\-]").unwrap().replace_all(name, "_").to_string().to_lowercase();
        let mut path = PathBuf::from(SERVER_DIRECTORY).join(&dir_name);
        let mut index = 1u32;
        loop {
            if !path.exists() {
                return dir_name;
            }
            path.set_file_name(format!("{} ({})", dir_name, index));
            index += 1;
        }
    }
}
