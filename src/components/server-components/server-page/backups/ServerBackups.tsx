import {useEffect, useState} from "react";
import {Card, CardBody, Chip, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, SelectItem, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, Textarea, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Button} from "../../../extended/Button.tsx";
import {Select} from "../../../extended/Select.tsx";
import {Input} from "../../../extended/Input.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import Checkbox from "../../../extended/Checkbox.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";

interface BackupData
{
    id: string; // Git commit ID
    backup_type: number; // 0=Full, 1=Incremental, 2=WorldOnly
    file_size: number;
    created_at: number;
    description: string;
}

interface BackupSchedule
{
    id: number;
    server_id: number;
    interval_amount: number;
    interval_unit: string; // "hours", "days", or "weeks"
    backup_type: number;
    enabled: boolean;
    retention_days?: number;
    last_run?: number;
    next_run?: number;
    created_at: number;
    updated_at: number;
}

interface BackupSettings
{
    schedules: BackupSchedule[];
}

interface IgnoreEntry
{
    pattern: string;
    comment?: string;
}

export function ServerBackups()
{
    const {server} = useServer();
    const [backups, setBackups] = useState<BackupData[]>([]);
    const [settings, setSettings] = useState<BackupSettings | null>(null);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [deleting, setDeleting] = useState<string | null>(null);
    const [restoring, setRestoring] = useState<string | null>(null);

    // Modal states
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const {isOpen: isScheduleOpen, onOpen: onScheduleOpen, onClose: onScheduleClose} = useDisclosure();
    const {isOpen: isRestoreOpen, onOpen: onRestoreOpen, onClose: onRestoreClose} = useDisclosure();
    const {isOpen: isIgnoreOpen, onOpen: onIgnoreOpen, onClose: onIgnoreClose} = useDisclosure();

    const [createDescription, setCreateDescription] = useState("");
    const [createBackupType, setCreateBackupType] = useState("0");
    const [restoreBackupId, setRestoreBackupId] = useState<string | null>(null);

    // Schedule form state
    const [scheduleForm, setScheduleForm] = useState({
        intervalAmount: 6,
        intervalUnit: "hours" as "hours" | "days" | "weeks",
        backupType: 0,
        enabled: true,
        retentionDays: 7
    });

    // Ignore management state
    const [ignoreEntries, setIgnoreEntries] = useState<IgnoreEntry[]>([]);
    const [loadingIgnore, setLoadingIgnore] = useState(false);
    const [savingIgnore, setSavingIgnore] = useState(false);

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
                setBackups(data);
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
            } else
            {
                console.error("Failed to load backup settings");
            }
        } catch (error)
        {
            console.error("Error loading backup settings:", error);
        }
    };

    const loadIgnoreList = async () =>
    {
        setLoadingIgnore(true);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/ignore`);
            if (response.ok)
            {
                const data = await response.json();
                setIgnoreEntries(data.entries || []);
            }
        } catch (error)
        {
            console.error("Error loading ignore list:", error);
        } finally
        {
            setLoadingIgnore(false);
        }
    };

    const saveIgnoreList = async () =>
    {
        setSavingIgnore(true);
        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/ignore`, {
                method: "PUT",
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify({entries: ignoreEntries})
            });

            if (response.ok)
            {
                onIgnoreClose();
            } else
            {
                console.error("Failed to save ignore list");
            }
        } catch (error)
        {
            console.error("Error saving ignore list:", error);
        } finally
        {
            setSavingIgnore(false);
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
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify({
                    backup_type: parseInt(createBackupType),
                    description: createDescription || null
                })
            });

            if (response.ok)
            {
                await loadBackups();
                onCreateClose();
                setCreateDescription("");
                setCreateBackupType("0");
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

    const createSchedule = async () =>
    {
        if (!server?.id) return;

        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/settings`, {
                method: "PUT",
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify({
                    interval_amount: scheduleForm.intervalAmount,
                    interval_unit: scheduleForm.intervalUnit,
                    backup_type: scheduleForm.backupType,
                    enabled: scheduleForm.enabled,
                    retention_days: scheduleForm.retentionDays
                })
            });

            if (response.ok)
            {
                await loadSettings();
                onScheduleClose();
            } else
            {
                const error = await response.json();
                console.error("Failed to create schedule:", error);
            }
        } catch (error)
        {
            console.error("Error creating schedule:", error);
        }
    };

    const deleteSchedule = async (scheduleId: number) =>
    {
        if (!server?.id) return;

        try
        {
            const response = await fetch(`/api/server/${server?.id}/backups/schedules/${scheduleId}`, {
                method: "DELETE"
            });

            if (response.ok)
            {
                await loadSettings();
            }
        } catch (error)
        {
            console.error("Error deleting schedule:", error);
        }
    };

    const downloadBackup = (backupId: string) =>
    {
        const link = document.createElement('a');
        link.href = `/api/server/${server?.id}/backups/${backupId}/download`;
        link.target = '_blank';
        link.click();
    };

    const openRestoreConfirmation = (backupId: string) =>
    {
        setRestoreBackupId(backupId);
        onRestoreOpen();
    };

    const openIgnoreModal = () =>
    {
        loadIgnoreList();
        onIgnoreOpen();
    };

    const addIgnoreEntry = () =>
    {
        setIgnoreEntries([...ignoreEntries, {pattern: "", comment: ""}]);
    };

    const removeIgnoreEntry = (index: number) =>
    {
        setIgnoreEntries(ignoreEntries.filter((_, i) => i !== index));
    };

    const updateIgnoreEntry = (index: number, field: "pattern" | "comment", value: string) =>
    {
        const newEntries = [...ignoreEntries];
        newEntries[index][field] = value;
        setIgnoreEntries(newEntries);
    };

    const formatDate = (timestamp: number) =>
    {
        return new Date(timestamp * 1000).toLocaleString();
    };

    const formatSize = (bytes: number) =>
    {
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    };

    const getBackupTypeName = (type: number) =>
    {
        switch(type) {
            case 0: return "Full";
            case 1: return "Incremental";
            case 2: return "World Only";
            default: return "Unknown";
        }
    };

    const getScheduleInterval = (schedule: BackupSchedule): {amount: number, unit: string} =>
    {
        return {
            amount: schedule.interval_amount,
            unit: schedule.interval_unit
        };
    };

    if (loading)
    {
        return (
            <div className="flex justify-center items-center h-64">
                <Spinner size="lg"/>
            </div>
        );
    }

    const activeSchedules = settings?.schedules.filter(s => s.enabled) || [];

    return (
        <div className="flex flex-col gap-4 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative font-minecraft-body">
            {/* Header with actions */}
            <div className="flex justify-between items-center">
                <div>
                    <h2 className="text-2xl">Server Backups</h2>
                    <p className="text-default-500">
                        {backups.length} backup{backups.length !== 1 ? "s" : ""} available
                        {activeSchedules.length > 0 && (
                            <Chip size="sm" color="success" variant="flat" className="ml-2">
                                {activeSchedules.length} Active Schedule{activeSchedules.length !== 1 ? "s" : ""}
                            </Chip>
                        )}
                    </p>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="flat"
                        onPress={openIgnoreModal}
                        startContent={<Icon icon="pixelarticons:eye-closed"/>}
                    >
                        Ignored Files
                    </Button>
                    <Button
                        variant="flat"
                        onPress={onScheduleOpen}
                        startContent={<Icon icon="pixelarticons:calendar"/>}
                    >
                        Schedule
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

            {/* Active Schedules */}
            {activeSchedules.length > 0 && (
                <Card>
                    <CardBody>
                        <h3 className="text-lg mb-2">Active Schedules</h3>
                        <div className="flex flex-col gap-2">
                            {activeSchedules.map(schedule => {
                                const interval = getScheduleInterval(schedule);
                                return (
                                    <div key={schedule.id} className="flex items-center justify-between p-2 bg-default-100 rounded-lg">
                                        <div className="flex items-center gap-3">
                                            <Icon icon="pixelarticons:clock" className="text-primary"/>
                                            <div>
                                                <p className="text-sm font-medium">
                                                    Every {interval.amount} {interval.unit}
                                                </p>
                                                <p className="text-xs text-default-500">
                                                    {getBackupTypeName(schedule.backup_type)} backup •
                                                    Keep {schedule.retention_days || 7} days
                                                </p>
                                            </div>
                                        </div>
                                        <Button
                                            size="sm"
                                            color="danger"
                                            variant="flat"
                                            isIconOnly
                                            onPress={() => deleteSchedule(schedule.id)}
                                        >
                                            <Icon icon="pixelarticons:trash"/>
                                        </Button>
                                    </div>
                                );
                            })}
                        </div>
                    </CardBody>
                </Card>
            )}

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
                            <TableColumn>Commit ID</TableColumn>
                            <TableColumn>Type</TableColumn>
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
                                <TableRow key={backup.id}>
                                    <TableCell>
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:git-commit" className="text-default-400"/>
                                            <span className="text-sm font-mono">{backup.id.substring(0, 8)}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <Chip size="sm" variant="flat">
                                            {getBackupTypeName(backup.backup_type)}
                                        </Chip>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{formatSize(backup.file_size)}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{formatDate(backup.created_at)}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm text-default-500">
                                            {backup.description}
                                        </span>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-1">
                                            <Tooltip content="Download this backup">
                                                <Button
                                                    size="sm"
                                                    color="primary"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => downloadBackup(backup.id)}
                                                >
                                                    <Icon icon="pixelarticons:download"/>
                                                </Button>
                                            </Tooltip>
                                            <Tooltip content="Restore this backup">
                                                <Button
                                                    size="sm"
                                                    color="warning"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => openRestoreConfirmation(backup.id)}
                                                    isLoading={restoring === backup.id}
                                                >
                                                    <Icon icon="pixelarticons:reply-all"/>
                                                </Button>
                                            </Tooltip>
                                            <Tooltip content="Delete this backup">
                                                <Button
                                                    size="sm"
                                                    color="danger"
                                                    variant="flat"
                                                    isIconOnly
                                                    onPress={() => deleteBackup(backup.id)}
                                                    isLoading={deleting === backup.id}
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
                classNames={{closeButton: "rounded-none"}}
            >
                <ModalContent>
                    <ModalHeader>Create New Backup</ModalHeader>
                    <ModalBody>
                        <Select
                            label="Backup Type"
                            selectedKeys={[createBackupType]}
                            onSelectionChange={(keys) => setCreateBackupType(Array.from(keys)[0] as string)}
                        >
                            <SelectItem key="0">Full Backup</SelectItem>
                            <SelectItem key="1">Incremental Backup</SelectItem>
                            <SelectItem key="2">World Only</SelectItem>
                        </Select>
                        <Textarea
                            label="Description (Optional)"
                            placeholder="Enter a description for this backup..."
                            value={createDescription}
                            onValueChange={setCreateDescription}
                            maxRows={3}
                            radius="none"
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

            {/* Schedule Modal */}
            <Modal
                isOpen={isScheduleOpen}
                onClose={onScheduleClose}
                size="lg"
                backdrop="blur"
                radius="none"
            >
                <ModalContent>
                    <ModalHeader>Create Backup Schedule</ModalHeader>
                    <ModalBody className="gap-4">
                        <div className="flex items-center gap-3 p-3 bg-primary-50 rounded-lg border border-primary-200">
                            <Icon icon="pixelarticons:info" className="text-primary-600 text-xl"/>
                            <div>
                                <p className="text-sm font-medium text-primary-800">
                                    Automated Backups
                                </p>
                                <p className="text-sm text-primary-600">
                                    Schedule automatic backups to run at regular intervals.
                                </p>
                            </div>
                        </div>

                        <div className="flex gap-2">
                            <Input
                                label="Every"
                                type="number"
                                min={1}
                                value={scheduleForm.intervalAmount.toString()}
                                onValueChange={(v) => setScheduleForm({...scheduleForm, intervalAmount: parseInt(v) || 1})}
                                className="w-1/3"
                            />
                            <Select
                                label="Unit"
                                selectedKeys={[scheduleForm.intervalUnit]}
                                onSelectionChange={(keys) => setScheduleForm({...scheduleForm, intervalUnit: Array.from(keys)[0] as any})}
                                className="w-2/3"
                            >
                                <SelectItem key="hours">Hours</SelectItem>
                                <SelectItem key="days">Days</SelectItem>
                                <SelectItem key="weeks">Weeks</SelectItem>
                            </Select>
                        </div>

                        <Select
                            label="Backup Type"
                            selectedKeys={[scheduleForm.backupType.toString()]}
                            onSelectionChange={(keys) => setScheduleForm({...scheduleForm, backupType: parseInt(Array.from(keys)[0] as string)})}
                        >
                            <SelectItem key="0">Full Backup</SelectItem>
                            <SelectItem key="1">Incremental Backup</SelectItem>
                            <SelectItem key="2">World Only</SelectItem>
                        </Select>

                        <Input
                            label="Retention Days"
                            type="number"
                            min={1}
                            value={scheduleForm.retentionDays.toString()}
                            onValueChange={(v) => setScheduleForm({...scheduleForm, retentionDays: parseInt(v) || 7})}
                            description="Number of days to keep backups (older backups will be deleted)"
                        />

                        <Checkbox
                            label="Enable Schedule"
                            checked={scheduleForm.enabled}
                            onChange={(enabled) => setScheduleForm({...scheduleForm, enabled})}
                            labelPlacement="left"
                        />
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="flat" onPress={onScheduleClose}>
                            Cancel
                        </Button>
                        <Button
                            color="primary"
                            onPress={createSchedule}
                        >
                            Create Schedule
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Manage Ignored Files Modal */}
            <Modal
                isOpen={isIgnoreOpen}
                onClose={onIgnoreClose}
                size="2xl"
                backdrop="blur"
                radius="none"
                scrollBehavior="inside"
            >
                <ModalContent>
                    <ModalHeader>Manage Ignored Files</ModalHeader>
                    <ModalBody>
                        <div className="flex items-center gap-3 p-3 bg-warning-50 rounded-lg border border-warning-200 mb-4">
                            <Icon icon="pixelarticons:info" className="text-warning-600 text-xl"/>
                            <div>
                                <p className="text-sm font-medium text-warning-800">
                                    .obakignore Configuration
                                </p>
                                <p className="text-sm text-warning-600">
                                    Files and directories matching these patterns will be excluded from backups.
                                </p>
                            </div>
                        </div>

                        {loadingIgnore ? (
                            <div className="flex justify-center py-8">
                                <Spinner/>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-3">
                                {ignoreEntries.map((entry, index) => (
                                    <div key={index} className="flex gap-2 items-start">
                                        <Input
                                            placeholder="Pattern (e.g., *.log, cache/, temp/**)"
                                            value={entry.pattern}
                                            onValueChange={(v) => updateIgnoreEntry(index, "pattern", v)}
                                            className="flex-1"
                                            size="sm"
                                        />
                                        <Input
                                            placeholder="Comment (optional)"
                                            value={entry.comment || ""}
                                            onValueChange={(v) => updateIgnoreEntry(index, "comment", v)}
                                            className="flex-1"
                                            size="sm"
                                        />
                                        <Button
                                            size="sm"
                                            color="danger"
                                            variant="flat"
                                            isIconOnly
                                            onPress={() => removeIgnoreEntry(index)}
                                        >
                                            <Icon icon="pixelarticons:trash"/>
                                        </Button>
                                    </div>
                                ))}

                                <Button
                                    variant="flat"
                                    onPress={addIgnoreEntry}
                                    startContent={<Icon icon="pixelarticons:plus"/>}
                                    className="mt-2"
                                >
                                    Add Pattern
                                </Button>

                                <div className="mt-4 p-3 bg-default-100 rounded-lg">
                                    <p className="text-sm font-medium mb-2">Pattern Examples:</p>
                                    <ul className="text-xs text-default-600 space-y-1">
                                        <li>• <code>*.log</code> - Ignore all .log files</li>
                                        <li>• <code>cache/</code> - Ignore the cache directory</li>
                                        <li>• <code>temp/**</code> - Ignore temp directory and all contents</li>
                                        <li>• <code>logs/*.txt</code> - Ignore .txt files in logs folder</li>
                                    </ul>
                                </div>
                            </div>
                        )}
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="flat" onPress={onIgnoreClose}>
                            Cancel
                        </Button>
                        <Button
                            color="primary"
                            onPress={saveIgnoreList}
                            isLoading={savingIgnore}
                        >
                            Save
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Restore Confirmation Modal */}
            <Modal
                isOpen={isRestoreOpen}
                onClose={onRestoreClose}
                radius="none"
                backdrop="blur"
                scrollBehavior="inside"
                closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
                classNames={{closeButton: "rounded-none"}}
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
                                        Restoring will replace all current server files with the backup data.
                                    </p>
                                </div>
                            </div>
                            <p className="text-default-600">
                                Are you sure you want to restore from this backup? This will:
                            </p>
                            <ul className="list-disc list-inside text-sm text-default-600 ml-4">
                                <li>Stop the server if it's currently running</li>
                                <li>Replace all server files with the backup data</li>
                                <li>You may need to restart the server manually</li>
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
        </div>
    );
}
