use serde_hash::HashIds;

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
    pub max_memory: i64,
    /// Minimum memory in GB for JVM -Xms argument
    pub min_memory: i64,
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
    pub backup_interval: i64,
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
    pub created_at: i64,
    /// Timestamp of when the server was last updated (seconds since epoch)
    pub updated_at: i64,
    /// Timestamp of when the server was last started (seconds since epoch)
    pub last_started: Option<i64>,
}
