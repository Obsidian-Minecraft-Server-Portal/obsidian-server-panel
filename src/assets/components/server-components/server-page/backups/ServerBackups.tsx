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
    file_size_formatted: string;
    created_at: number;
    created_at_formatted: string;
    description?: string;
}

interface BackupSettings
{
    backup_enabled: boolean;
    backup_cron: string;
    backup_type: string;
    backup_retention: number;
    is_scheduled: boolean;
}

export function ServerBackups()
{
    const {server} = useServer();
    const [backups, setBackups] = useState<BackupData[]>([]);
    const [settings, setSettings] = useState<BackupSettings | null>(null);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [deleting, setDeleting] = useState<string | null>(null);
    const [savingSettings, setSavingSettings] = useState(false);

    // Modal states
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const {isOpen: isSettingsOpen, onOpen: onSettingsOpen, onClose: onSettingsClose} = useDisclosure();
    const [createDescription, setCreateDescription] = useState("");

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

    const getBackupTypeColor = (type: string) =>
    {
        switch (type.toLowerCase())
        {
            case "full":
                return "primary";
            case "incremental":
                return "secondary";
            case "world":
                return "success";
            default:
                return "default";
        }
    };

    // const formatDate = (timestamp: number) => {
    //     return new Date(timestamp * 1000).toLocaleString();
    // };

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
                        color="secondary"
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
                                            <Icon icon="pixelarticons:archive" className="text-default-400"/>
                                            <span className="font-mono text-sm">{backup.filename}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <Chip
                                            size="sm"
                                            color={getBackupTypeColor(backup.backup_type) as any}
                                            variant="flat"
                                            className="capitalize"
                                        >
                                            {backup.backup_type}
                                        </Chip>
                                    </TableCell>
                                    <TableCell>
                                        <span className="font-mono text-sm">{backup.file_size_formatted}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{backup.created_at_formatted}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm text-default-500">
                                            {backup.description || "No description"}
                                        </span>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-1">
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
                                        </div>
                                    </TableCell>
                                </TableRow>
                            )}
                        </TableBody>
                    </Table>
                </CardBody>
            </Card>

            {/* Create Backup Modal */}
            <Modal isOpen={isCreateOpen} onClose={onCreateClose}>
                <ModalContent>
                    <ModalHeader>Create New Backup</ModalHeader>
                    <ModalBody>
                        <Textarea
                            label="Description (Optional)"
                            placeholder="Enter a description for this backup..."
                            value={createDescription}
                            onValueChange={setCreateDescription}
                            maxRows={3}
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
                                <Select
                                    label="Backup Type"
                                    selectedKeys={[formSettings.backup_type]}
                                    onSelectionChange={(keys) =>
                                        setFormSettings({
                                            ...formSettings,
                                            backup_type: Array.from(keys)[0] as string
                                        })
                                    }
                                >
                                    <SelectItem key="full">Full Backup</SelectItem>
                                    <SelectItem key="incremental">Incremental Backup</SelectItem>
                                    <SelectItem key="world">World Only</SelectItem>
                                </Select>
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
        </div>
    );
}