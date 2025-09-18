import {useEffect, useState} from "react";
import {Button, Card, CardBody, CheckboxGroup, Divider, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {motion} from "framer-motion";
import $ from "jquery";
import {PermissionFlag, UpdateUserRequest, UserData} from "../../types/UserTypes.ts";
import {MessageOptions, MessageResponseType} from "../MessageModal.tsx";
import Checkbox from "../extended/Checkbox.tsx";

interface EditUserModalProps
{
    isOpen: boolean;
    onClose: () => void;
    user: UserData;
    permissions: PermissionFlag[];
    onUserUpdated: () => void;
    onShowMessage: (options: MessageOptions) => void;
}

export default function EditUserModal({
                                          isOpen,
                                          onClose,
                                          user,
                                          permissions,
                                          onUserUpdated,
                                          onShowMessage
                                      }: EditUserModalProps)
{
    const [username, setUsername] = useState("");
    const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
    const [isActive, setIsActive] = useState(true);
    const [loading, setLoading] = useState(false);
    const [errors, setErrors] = useState<{ [key: string]: string }>({});

    // Initialize form with user data
    useEffect(() =>
    {
        if (user)
        {
            setUsername(user.username);
            setSelectedPermissions(user.permissions.map(p => p.id.toString()));
            setIsActive(user.is_active);
        }
    }, [user]);

    const validateForm = () =>
    {
        const newErrors: { [key: string]: string } = {};

        if (!username.trim())
        {
            newErrors.username = "Username is required";
        } else if (username.length < 3)
        {
            newErrors.username = "Username must be at least 3 characters";
        } else if (!/^[a-zA-Z0-9_-]+$/.test(username))
        {
            newErrors.username = "Username can only contain letters, numbers, underscores, and hyphens";
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = async () =>
    {
        if (!validateForm()) return;

        setLoading(true);
        try
        {
            const permissionIds = selectedPermissions.map(id => parseInt(id));
            const requestData: UpdateUserRequest = {
                username: username.trim(),
                permissions: permissionIds,
                is_active: isActive
            };

            await $.ajax(`/api/auth/users/${user.id}`, {
                method: "PUT",
                data: JSON.stringify(requestData),
                contentType: "application/json"
            });

            onShowMessage({
                title: "Success",
                body: `User "${username}" updated successfully.`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });

            onUserUpdated();
            onClose();
        } catch (error: any)
        {
            console.error("Failed to update user:", error);
            const errorMessage = error.responseJSON?.message || "Failed to update user. Please try again.";
            onShowMessage({
                title: "Error",
                body: errorMessage,
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally
        {
            setLoading(false);
        }
    };

    const handleClose = () =>
    {
        setErrors({});
        onClose();
    };

    const isAdminSelected = selectedPermissions.includes("1"); // Admin permission ID is 1
    const hasChanges =
        username !== user.username ||
        isActive !== user.is_active ||
        JSON.stringify(selectedPermissions.sort()) !== JSON.stringify(user.permissions.map(p => p.id.toString()).sort());

    return (
        <Modal
            isOpen={isOpen}
            onClose={handleClose}
            size="2xl"
            scrollBehavior="inside"
            backdrop="blur"
            radius="none"
            closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
            classNames={{
                closeButton: "rounded-none"
            }}
            isDismissable={!loading}
        >
            <ModalContent>
                {() => (
                    <>
                        <ModalHeader className="flex flex-row items-center gap-2 text-2xl font-minecraft-body">
                            <Icon icon="pixelarticons:edit" className="text-3xl text-primary"/>
                            <span>Edit User: {user.username}</span>
                        </ModalHeader>
                        <ModalBody>
                            <motion.div
                                initial={{opacity: 0, y: 20}}
                                animate={{opacity: 1, y: 0}}
                                transition={{duration: 0.2}}
                                className="flex flex-col gap-4"
                            >
                                {/* User Info */}
                                <Card className="bg-default-50">
                                    <CardBody className="p-3">
                                        <div className="flex flex-col gap-2 text-sm font-minecraft-body">
                                            <div className="flex justify-between">
                                                <span className="text-default-600">User ID:</span>
                                                <span>{user.id}</span>
                                            </div>
                                            <div className="flex justify-between">
                                                <span className="text-default-600">Joined:</span>
                                                <span>{new Date(user.join_date).toLocaleDateString()}</span>
                                            </div>
                                            <div className="flex justify-between">
                                                <span className="text-default-600">Last Online:</span>
                                                <span>{new Date(user.last_online).toLocaleDateString()}</span>
                                            </div>
                                        </div>
                                    </CardBody>
                                </Card>

                                {/* Username Input */}
                                <Input
                                    label="Username"
                                    placeholder="Enter username"
                                    value={username}
                                    onValueChange={setUsername}
                                    radius="none"
                                    isInvalid={!!errors.username}
                                    errorMessage={errors.username}
                                    startContent={<Icon icon="pixelarticons:user"/>}
                                    classNames={{
                                        label: "font-minecraft-body",
                                        input: "font-minecraft-body"
                                    }}
                                />

                                {/* User Status */}
                                <div className="flex items-center justify-between">
                                    <Checkbox
                                        label={
                                            <div className="flex flex-col gap-1">
                                                <span className="text-sm font-minecraft-body font-semibold">User Status</span>
                                                <span className="text-xs text-default-500 font-minecraft-body">Disabled users cannot log in to the system</span>
                                            </div>
                                        }
                                        checked={isActive}
                                        onChange={setIsActive}
                                        color={"primary"}
                                        fullWidth={true}
                                        labelPlacement={"left"}
                                    />
                                </div>

                                <Divider/>

                                {/* Permissions Selection */}
                                <div className="flex flex-col gap-3">
                                    <h3 className="text-lg font-minecraft-body">Permissions</h3>

                                    {isAdminSelected && (
                                        <Card className="bg-warning-50 border-warning-200">
                                            <CardBody className="p-3">
                                                <div className="flex items-center gap-2">
                                                    <Icon icon="pixelarticons:warning-box" className="text-warning"/>
                                                    <span className="text-sm font-minecraft-body text-warning-700">
                                                        Admin permission grants access to all features. Other permissions will be ignored.
                                                    </span>
                                                </div>
                                            </CardBody>
                                        </Card>
                                    )}

                                    <CheckboxGroup
                                        value={selectedPermissions}
                                        onValueChange={setSelectedPermissions}
                                        classNames={{
                                            wrapper: "gap-2"
                                        }}
                                    >
                                        {permissions.map((permission) => (
                                            <Checkbox
                                                key={permission.id}
                                                checked={selectedPermissions.includes(permission.id.toString())}
                                                onChange={checked => setSelectedPermissions(prev => checked ? [...prev, permission.id.toString()] : prev.filter(id => id !== permission.id.toString()))}
                                                label={
                                                    <div className="flex flex-col">
                                                        <span>{permission.name}</span>
                                                        <span className="text-xs text-default-500">{getPermissionDescription(permission.name)}</span>
                                                    </div>
                                                }
                                                className={"data-[admin=true]:text-primary data-[admin=true]:font-semibold"}
                                                labelPlacement={"left"}
                                                fullWidth
                                                data-admin={permission.name.toLowerCase() === "admin"}
                                            />
                                        ))}
                                    </CheckboxGroup>
                                </div>

                                {user.needs_password_change && (
                                    <>
                                        <Divider/>
                                        <Card className="bg-warning-50 border-warning-200">
                                            <CardBody className="p-3">
                                                <div className="flex items-center gap-2">
                                                    <Icon icon="pixelarticons:key" className="text-warning"/>
                                                    <span className="text-sm font-minecraft-body text-warning-700">
                                                        This user is required to change their password on next login.
                                                    </span>
                                                </div>
                                            </CardBody>
                                        </Card>
                                    </>
                                )}
                            </motion.div>
                        </ModalBody>
                        <ModalFooter>
                            <Button
                                radius="none"
                                onPress={handleClose}
                                isDisabled={loading}
                            >
                                Cancel
                            </Button>
                            <Button
                                color="primary"
                                radius="none"
                                onPress={handleSubmit}
                                isLoading={loading}
                                isDisabled={!hasChanges}
                                startContent={!loading ? <Icon icon="pixelarticons:save"/> : null}
                            >
                                Save Changes
                            </Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}

function getPermissionDescription(permissionName: string): string
{
    const descriptions: { [key: string]: string } = {
        "None": "No special permissions",
        "Admin": "Full access to all features and settings",
        "CreateServer": "Can create new servers",
        "OperateServer": "Can start, stop, and configure servers",
        "CreateBackup": "Can create server backups",
        "RestoreBackup": "Can restore server backups",
        "DeleteBackups": "Can delete server backups",
        "UploadFiles": "Can upload files to servers",
        "DeleteFiles": "Can delete server files",
        "CreateFiles": "Can create new server files",
        "ModifyFiles": "Can edit server files",
        "ViewUsers": "Can view user list and information",
        "ManageUsers": "Can create, edit, and delete users",
        "ManagePermissions": "Can modify user permissions",
        "ManageSettings": "Can modify application settings"
    };
    return descriptions[permissionName] || "Permission description not available";
}
