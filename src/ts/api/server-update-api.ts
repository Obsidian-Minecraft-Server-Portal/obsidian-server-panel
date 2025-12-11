export interface UpdateInfo {
    update_available: boolean;
    current_version?: string;
    latest_version?: string;
    download_url?: string;
    changelog_url?: string;
    message?: string;
}

export interface BackupJarInfo {
    backups: string[];
}

/**
 * Check if updates are available for a server
 */
export async function checkForUpdates(serverId: string): Promise<UpdateInfo> {
    const response = await fetch(`/api/server/${serverId}/updates/check`);
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to check for updates');
    }
    return response.json();
}

/**
 * Apply available update to a server
 */
export async function applyUpdate(serverId: string): Promise<{ message: string; new_version: string }> {
    const response = await fetch(`/api/server/${serverId}/updates/apply`, {
        method: 'POST',
    });
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to apply update');
    }
    return response.json();
}

/**
 * Rollback to previous server version
 */
export async function rollbackUpdate(serverId: string): Promise<{ message: string }> {
    const response = await fetch(`/api/server/${serverId}/updates/rollback`, {
        method: 'POST',
    });
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to rollback update');
    }
    return response.json();
}

/**
 * List available backup jars
 */
export async function listBackupJars(serverId: string): Promise<BackupJarInfo> {
    const response = await fetch(`/api/server/${serverId}/updates/backups`);
    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to list backups');
    }
    return response.json();
}
