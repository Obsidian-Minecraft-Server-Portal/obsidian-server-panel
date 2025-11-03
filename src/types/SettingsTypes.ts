export interface Settings {
    general: GeneralSettings;
    network: NetworkSettings;
    storage: StorageSettings;
    java: JavaSettings;
}

export interface GeneralSettings {
    port: number;
    auto_start: boolean;
}

export interface NetworkSettings {
    auto_port_forward: boolean;
    upnp_enabled: boolean;
}

export interface StorageSettings {
    servers_directory: string;
    java_directory: string;
    backups_directory: string;
    temp_directory: string;
}

export interface JavaSettings {
    default_runtime?: string | null;
}

export interface ValidationResult {
    valid: boolean;
    message: string;
    error?: string;
}

export interface SettingsUpdateResponse {
    message: string;
    settings: Settings;
    migration_info?: string[];
}
