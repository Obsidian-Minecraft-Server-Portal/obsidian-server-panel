import { useState, useCallback } from 'react';
import {
    checkForUpdates,
    applyUpdate,
    rollbackUpdate,
    listBackupJars,
    type UpdateInfo
} from '../ts/api/server-update-api';

export function useServerUpdates(serverId: string) {
    const [checking, setChecking] = useState(false);
    const [applying, setApplying] = useState(false);
    const [rollingBack, setRollingBack] = useState(false);
    const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
    const [backupJars, setBackupJars] = useState<string[]>([]);
    const [error, setError] = useState<string | null>(null);

    const checkUpdates = useCallback(async () => {
        setChecking(true);
        setError(null);
        try {
            const info = await checkForUpdates(serverId);
            setUpdateInfo(info);
            return info;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to check for updates';
            setError(errorMessage);
            throw err;
        } finally {
            setChecking(false);
        }
    }, [serverId]);

    const applyServerUpdate = useCallback(async () => {
        setApplying(true);
        setError(null);
        try {
            const result = await applyUpdate(serverId);
            setUpdateInfo(null); // Clear update info after successful apply
            return result;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to apply update';
            setError(errorMessage);
            throw err;
        } finally {
            setApplying(false);
        }
    }, [serverId]);

    const rollbackServerUpdate = useCallback(async () => {
        setRollingBack(true);
        setError(null);
        try {
            const result = await rollbackUpdate(serverId);
            return result;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to rollback update';
            setError(errorMessage);
            throw err;
        } finally {
            setRollingBack(false);
        }
    }, [serverId]);

    const loadBackupJars = useCallback(async () => {
        try {
            const result = await listBackupJars(serverId);
            setBackupJars(result.backups);
            return result;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to list backup jars';
            setError(errorMessage);
            throw err;
        }
    }, [serverId]);

    return {
        checking,
        applying,
        rollingBack,
        updateInfo,
        backupJars,
        error,
        checkUpdates,
        applyServerUpdate,
        rollbackServerUpdate,
        loadBackupJars,
    };
}
