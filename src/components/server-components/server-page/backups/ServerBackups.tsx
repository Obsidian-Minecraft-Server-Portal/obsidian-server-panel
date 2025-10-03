import {useEffect, useState} from "react";
import {Card, CardBody, Chip, Link, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, SelectItem, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, Textarea, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Button} from "../../../extended/Button.tsx";
import {Select} from "../../../extended/Select.tsx";
import {Input} from "../../../extended/Input.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import Checkbox from "../../../extended/Checkbox.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";

interface BackupData
{
    id: string;
    server_id: string;
    filename: string;
    backup_type: string;
    file_size: number;
    created_at: number;
    description?: string;
    git_commit_id?: string; // For git-based incremental backups
}

interface BackupListResponse
{
    backup: BackupData;
    file_size_formatted: string;
    created_at_formatted: string;
}

interface BackupSettings
{
    backup_enabled: boolean;
    backup_cron: string;
    backup_retention: number;
    is_scheduled: boolean;
}

export function ServerBackups()
{
    const {server} = useServer();
    const [backups, setBackups] = useState<BackupListResponse[]>([]);
    const [settings, setSettings] = useState<BackupSettings | null>(null);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [deleting, setDeleting] = useState<string | null>(null);
    const [restoring, setRestoring] = useState<string | null>(null);
    const [savingSettings, setSavingSettings] = useState(false);

    // Modal states
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const {isOpen: isSettingsOpen, onOpen: onSettingsOpen, onClose: onSettingsClose} = useDisclosure();
    const {isOpen: isRestoreOpen, onOpen: onRestoreOpen, onClose: onRestoreClose} = useDisclosure();
    const {isOpen: isDownloadOpen, onOpen: onDownloadOpen, onClose: onDownloadClose} = useDisclosure();
    const [createDescription, setCreateDescription] = useState("");
    const [restoreBackupId, setRestoreBackupId] = useState<string | null>(null);
    const [downloadBackup, setDownloadBackup] = useState<BackupListResponse | null>(null);

    // Settings form state
    const [formSettings, setFormSettings] = useState<BackupSettings | null>(null);

    useEffect(() =>
    {
        if (server)
        {
            loadBackups();
            loadSettings();
        }
    }, [server]);

    const loadBackups = async () =>
    {
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups`);
            if (response.ok)
            {
                const data = await response.json();
                setBackups(data.backups || []);
            } else
            {
                console.error("Failed to load backups");
            }
        } catch (error)
        {
            console.error("Error loading backups:", error);
        } finally
        {
            setLoading(false);
        }
    };

    const loadSettings = async () =>
    {
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/settings`);
            if (response.ok)
            {
                const data = await response.json();
                setSettings(data);
                setFormSettings(data);
            } else
            {
                console.error("Failed to load backup settings");
            }
        } catch (error)
        {
            console.error("Error loading backup settings:", error);
        }
    };

    const createBackup = async () =>
    {
        if (!server?.id) return;

        setCreating(true);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    description: createDescription || undefined
                })
            });

            if (response.ok)
            {
                await loadBackups();
                onCreateClose();
                setCreateDescription("");
            } else
            {
                const error = await response.json();
                console.error("Failed to create backup:", error);
            }
        } catch (error)
        {
            console.error("Error creating backup:", error);
        } finally
        {
            setCreating(false);
        }
    };

    const deleteBackup = async (backupId: string) =>
    {
        if (!server?.id) return;

        setDeleting(backupId);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/${backupId}`, {
                method: "DELETE"
            });

            if (response.ok)
            {
                await loadBackups();
            } else
            {
                const error = await response.json();
                console.error("Failed to delete backup:", error);
            }
        } catch (error)
        {
            console.error("Error deleting backup:", error);
        } finally
        {
            setDeleting(null);
        }
    };

    const restoreBackup = async () =>
    {
        if (!server?.id || !restoreBackupId) return;

        setRestoring(restoreBackupId);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/${restoreBackupId}/restore`, {
                method: "POST"
            });

            if (response.ok)
            {
                onRestoreClose();
                setRestoreBackupId(null);
                // Show success message or refresh data
            } else
            {
                const error = await response.json();
                console.error("Failed to restore backup:", error);
            }
        } catch (error)
        {
            console.error("Error restoring backup:", error);
        } finally
        {
            setRestoring(null);
        }
    };

    const openRestoreConfirmation = (backupId: string) =>
    {
        setRestoreBackupId(backupId);
        onRestoreOpen();
    };

    const handleDownloadClick = (backup: BackupListResponse) =>
    {
        // Check if it's a git-based backup
        if (backup.backup.git_commit_id) {
            // Show download options modal for git-based backups
            setDownloadBackup(backup);
            onDownloadOpen();
        } else {
            // Direct download for legacy ZIP-based backups
            const link = document.createElement('a');
            link.href = `/api/server/${server?.id}/backups/${backup.backup.id}/download`;
            link.target = '_blank';
            link.click();
        }
    };

    const handleDownloadOption = (downloadType: string) =>
    {
        if (!downloadBackup) return;
        
        const link = document.createElement('a');
        link.href = `/api/server/${server?.id}/backups/${downloadBackup.backup.id}/download?type=${downloadType}`;
        link.target = '_blank';
        link.click();
        
        onDownloadClose();
        setDownloadBackup(null);
    };

    const saveSettings = async () =>
    {
        if (!server?.id || !formSettings) return;

        setSavingSettings(true);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/settings`, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(formSettings)
            });

            if (response.ok)
            {
                await loadSettings();
                onSettingsClose();
            } else
            {
                const error = await response.json();
                console.error("Failed to save backup settings:", error);
            }
        } catch (error)
        {
            console.error("Error saving backup settings:", error);
        } finally
        {
            setSavingSettings(false);
        }
    };

    if (loading)
    {
        return (
            <div className="flex justify-center items-center h-64">
                <Spinner size="lg"/>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative font-minecraft-body">
            {/* Header with actions */}
            <div className="flex justify-between items-center">
                <div>
                    <h2 className="text-2xl">Server Backups</h2>
                    <p className="text-default-500">
                        {backups.length} backup{backups.length !== 1 ? "s" : ""} available
                        {settings?.is_scheduled && (
                            <Chip size="sm" color="success" variant="flat" className="ml-2">
                                Scheduled
                            </Chip>
                        )}
                    </p>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="flat"
                        onPress={onSettingsOpen}
                        startContent={<Icon icon="pixelarticons:settings"/>}
                    >
                        Settings
                    </Button>
                    <Button
                        color="primary"
                        onPress={onCreateOpen}
                        isLoading={creating}
                        startContent={<Icon icon="pixelarticons:plus"/>}
                    >
                        Create Backup
                    </Button>
                </div>
            </div>

            {/* Backups Table */}
            <Card className="flex-1">
                <CardBody className="p-0">
                    <Table
                        aria-label="Server backups table"
                        isHeaderSticky
                        classNames={{
                            wrapper: "max-h-[400px]"
                        }}
                        removeWrapper
                    >
                        <TableHeader>
                            <TableColumn>Filename</TableColumn>
                            <TableColumn>Size</TableColumn>
                            <TableColumn>Created</TableColumn>
                            <TableColumn>Description</TableColumn>
                            <TableColumn>Actions</TableColumn>
                        </TableHeader>
                        <TableBody
                            emptyContent="No backups found"
                            items={backups}
                        >
                            {(backup) => (
                                <TableRow key={backup.backup.id}>
                                    <TableCell>
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:archive" className="text-default-400"/>
                                            <span className="text-sm">{backup.backup.filename}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{backup.file_size_formatted}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{backup.created_at_formatted}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm text-default-500">
                                            {backup.backup.description || "No description"}
                                        </span>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-1">
                                            <Tooltip content={"Download this backup"}>
                                                <Button
                                                    size="sm"
                                                    color="primary"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => handleDownloadClick(backup)}
                                                >
                                                    <Icon icon="pixelarticons:download"/>
                                                </Button>
                                            </Tooltip>
                                            <Tooltip content={"Restore this backup"}>
                                                <Button
                                                    size="sm"
                                                    color="warning"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => openRestoreConfirmation(backup.backup.id)}
                                                    isLoading={restoring === backup.backup.id}
                                                >
                                                    <Icon icon="pixelarticons:reply-all"/>
                                                </Button>
                                            </Tooltip>
                                            <Tooltip content={"Delete this backup"}>
                                                <Button
                                                    size="sm"
                                                    color="danger"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => deleteBackup(backup.backup.id)}
                                                    isLoading={deleting === backup.backup.id}
                                                >
                                                    <Icon icon="pixelarticons:trash"/>
                                                </Button>
                                            </Tooltip>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            )}
                        </TableBody>
                    </Table>
                </CardBody>
            </Card>

            {/* Create Backup Modal */}
            <Modal
                isOpen={isCreateOpen}
                onClose={onCreateClose}
                scrollBehavior="inside"
                backdrop="blur"
                radius="none"
                closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
                classNames={{
                    closeButton: "rounded-none"
                }}
            >
                <ModalContent>
                    <ModalHeader>Create New Backup</ModalHeader>
                    <ModalBody>
                        <Textarea
                            label="Description (Optional)"
                            placeholder="Enter a description for this backup..."
                            value={createDescription}
                            onValueChange={setCreateDescription}
                            maxRows={3}
                            radius={"none"}
                        />
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="flat" onPress={onCreateClose}>
                            Cancel
                        </Button>
                        <Button
                            color="primary"
                            onPress={createBackup}
                            isLoading={creating}
                        >
                            Create Backup
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Settings Modal */}
            <Modal isOpen={isSettingsOpen} onClose={onSettingsClose} size="lg" className={"font-minecraft-body"} radius={"none"} backdrop={"blur"}>
                <ModalContent>
                    <ModalHeader>Backup Settings</ModalHeader>
                    <ModalBody className="gap-4">
                        {formSettings && (
                            <>
                                <div className="flex items-center gap-3 p-3 bg-primary-50 rounded-lg border border-primary-200">
                                    <Icon icon="pixelarticons:info" className="text-primary-600 text-xl"/>
                                    <div>
                                        <p className="text-sm font-medium text-primary-800">
                                            Incremental Backups Only
                                        </p>
                                        <p className="text-sm text-primary-600">
                                            All backups are created using efficient incremental Git-based storage.
                                        </p>
                                    </div>
                                </div>
                                <div className={"flex flex-row gap-2 items-start"}>
                                    <Select
                                        label={"Backup Schedule"}
                                        description="Cron expression for backup schedule (e.g., '0 0 * * * *' for hourly)"
                                        selectedKeys={formSettings.backup_cron}
                                        onSelectionChange={
                                            (key) =>
                                            {
                                                if (!key) return;
                                                setFormSettings({
                                                    ...formSettings,
                                                    backup_cron: key as string,
                                                    is_scheduled: true
                                                });
                                            }
                                        }
                                        size={"sm"}
                                    >
                                        <SelectItem key={"0 */15 * * * *"}>Every 15 Minutes</SelectItem>
                                        <SelectItem key={"0 */30 * * * *"}>Every 30 Minutes</SelectItem>
                                        <SelectItem key={"0 0 * * * *"}>Every Hour</SelectItem>
                                        <SelectItem key={"0 0 */2 * * *"}>Every Other Hour</SelectItem>
                                        <SelectItem key={"0 0 */6 * * *"}>Every 6 Hours</SelectItem>
                                        <SelectItem key={"0 0 0 * * *"}>Every Day</SelectItem>
                                        <SelectItem key={"0 0 0 */2 * *"}>Every Other Day</SelectItem>
                                        <SelectItem key={"0 0 0 */14 * *"}>BiWeekly</SelectItem>
                                        <SelectItem key={"0 0 0 1 * *"}>1st of Every Month</SelectItem>
                                        <SelectItem key={"0 0 0 1 */2 *"}>Every Other Month</SelectItem>
                                        <SelectItem key={"0 0 0 1 */6 *"}>Every 6 Months</SelectItem>
                                        <SelectItem key={"0 0 0 1 1 *"}>Every Year</SelectItem>

                                    </Select>

                                    <Tooltip content={"Open Cron Expression Generator"}>
                                        <Button isIconOnly as={Link} href={"https://crontab.cronhub.io/"} target={"_blank"} size={"lg"}><Icon icon={"pixelarticons:external-link"}/></Button>
                                    </Tooltip>
                                </div>

                                <Input
                                    label="Backup Retention"
                                    type="number"
                                    min={1}
                                    max={100}
                                    value={formSettings.backup_retention.toString()}
                                    onValueChange={(retention) =>
                                        setFormSettings({
                                            ...formSettings,
                                            backup_retention: parseInt(retention) || 7
                                        })
                                    }
                                    description="Number of backups to keep (older backups will be deleted)"
                                />
                                <Checkbox
                                    label={"Enable Automatic Backups"}
                                    checked={formSettings.backup_enabled}
                                    onChange={(enabled) =>
                                        setFormSettings({...formSettings, backup_enabled: enabled})
                                    }
                                    labelPlacement={"left"}
                                />
                            </>
                        )}
                    </ModalBody>
                    <ModalFooter>
                        <Button
                            color="primary"
                            onPress={saveSettings}
                            isLoading={savingSettings}
                        >
                            Save Settings
                        </Button>
                        <Button variant="flat" onPress={onSettingsClose}>
                            Cancel
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Restore Confirmation Modal */}
            <Modal
                isOpen={isRestoreOpen}
                onClose={onRestoreClose}
                className={"font-minecraft-body"}
                radius={"none"}
                backdrop={"blur"}
                scrollBehavior="inside"
                closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
                classNames={{
                    closeButton: "rounded-none"
                }}
            >
                <ModalContent>
                    <ModalHeader>Restore Backup</ModalHeader>
                    <ModalBody>
                        <div className="flex flex-col gap-4">
                            <div className="flex items-center gap-3 p-3 bg-warning-50 rounded-lg border border-warning-200">
                                <Icon icon="pixelarticons:warning" className="text-warning-600 text-xl"/>
                                <div>
                                    <p className="font-semibold text-warning-800">Warning: This action cannot be undone</p>
                                    <p className="text-sm text-warning-700">
                                        Restoring this backup will replace all current server files. Your current server data will be backed up before restoration.
                                    </p>
                                </div>
                            </div>
                            <p className="text-default-600">
                                Are you sure you want to restore from this backup? This will:
                            </p>
                            <ul className="list-disc list-inside text-sm text-default-600 ml-4">
                                <li>Stop the server if it's currently running</li>
                                <li>Create a backup of your current server files</li>
                                <li>Replace all server files with the backup data</li>
                                <li>You may need to restart the server manually after restoration</li>
                            </ul>
                        </div>
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="flat" onPress={onRestoreClose}>
                            Cancel
                        </Button>
                        <Button
                            color="warning"
                            onPress={restoreBackup}
                            isLoading={restoring === restoreBackupId}
                        >
                            Restore Backup
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Download Options Modal for Git-based Incremental Backups */}
            <Modal
                isOpen={isDownloadOpen}
                onClose={onDownloadClose}
                className={"font-minecraft-body"}
                radius={"none"}
                backdrop={"blur"}
                scrollBehavior="inside"
                closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
                classNames={{
                    closeButton: "rounded-none"
                }}
            >
                <ModalContent>
                    <ModalHeader>Download Backup Options</ModalHeader>
                    <ModalBody>
                        <div className="flex flex-col gap-4">
                            <div className="flex items-center gap-3 p-3 bg-primary-50 rounded-lg border border-primary-200">
                                <Icon icon="pixelarticons:info" className="text-primary-600 text-xl"/>
                                <div>
                                    <p className="text-sm font-medium text-primary-800">
                                        Git-based Incremental Backup
                                    </p>
                                    <p className="text-sm text-primary-600">
                                        Choose how you want to download this backup.
                                    </p>
                                </div>
                            </div>

                            <div className="space-y-3">
                                <div className="p-3 border border-default-300 rounded-lg">
                                    <h4 className="font-medium text-sm mb-1">Modified Files Only (Diff)</h4>
                                    <p className="text-xs text-default-500 mb-3">
                                        Download only the files that were changed in this backup compared to the previous backup.
                                        Smaller download size, but requires manual merging.
                                    </p>
                                    <Button
                                        size="sm"
                                        color="primary"
                                        variant="flat"
                                        onPress={() => handleDownloadOption("diff")}
                                        startContent={<Icon icon="pixelarticons:diff"/>}
                                    >
                                        Download Changes Only
                                    </Button>
                                </div>

                                <div className="p-3 border border-default-300 rounded-lg">
                                    <h4 className="font-medium text-sm mb-1">Full Backup From This Point</h4>
                                    <p className="text-xs text-default-500 mb-3">
                                        Download a complete backup containing all files as they were at this backup point.
                                        Larger download size, but ready to restore directly.
                                    </p>
                                    <Button
                                        size="sm"
                                        color="success"
                                        variant="flat"
                                        onPress={() => handleDownloadOption("full")}
                                        startContent={<Icon icon="pixelarticons:archive"/>}
                                    >
                                        Download Full Backup
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="flat" onPress={onDownloadClose}>
                            Cancel
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>
        </div>
    );
}