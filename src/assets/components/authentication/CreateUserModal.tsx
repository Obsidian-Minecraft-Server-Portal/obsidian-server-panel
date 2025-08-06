import {useState} from "react";
import {Button, Card, CardBody, Checkbox, CheckboxGroup, Code, Divider, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {motion} from "framer-motion";
import $ from "jquery";
import {CreateUserRequest, PermissionFlag} from "../../types/UserTypes.ts";
import {MessageOptions, MessageResponseType} from "../MessageModal.tsx";

type CreateUserRepsonse = {
    message: string;
    user_id: number;
    username: string;
    password_change_required: boolean;
    password: string;
};

interface CreateUserModalProps
{
    isOpen: boolean;
    onClose: () => void;
    permissions: PermissionFlag[];
    onUserCreated: () => void;
    onShowMessage: (options: MessageOptions) => void;
}

export default function CreateUserModal({
                                            isOpen,
                                            onClose,
                                            permissions,
                                            onUserCreated,
                                            onShowMessage
                                        }: CreateUserModalProps)
{
    const [username, setUsername] = useState("");
    const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
    const [loading, setLoading] = useState(false);
    const [errors, setErrors] = useState<{ [key: string]: string }>({});

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
            const requestData: CreateUserRequest = {
                username: username.trim(),
                permissions: permissionIds
            };

            const response: CreateUserRepsonse = await $.ajax("/api/auth/users", {
                method: "POST",
                data: JSON.stringify(requestData),
                contentType: "application/json"
            });

            onShowMessage({
                title: "Success",
                body: (
                    <>
                        <p>
                            User "{response.username}" created successfully. A random password has been generated and the user will be required to change it on first login.
                        </p>
                        <Code
                            radius={"none"}
                            onClick={() =>
                            {
                                navigator.clipboard.writeText(response.password);
                            }}
                            className={"cursor-pointer"}
                        >
                            {response.password}
                        </Code>
                    </>
                ),
                responseType: MessageResponseType.Close,
                severity: "success"
            });

            // Reset form
            setUsername("");
            setSelectedPermissions([]);
            setErrors({});
            onUserCreated();
            onClose();
        } catch (error: any)
        {
            console.error("Failed to create user:", error);
            const errorMessage = error.responseJSON?.message || "Failed to create user. Please try again.";
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
        setUsername("");
        setSelectedPermissions([]);
        setErrors({});
        onClose();
    };

    const isAdminSelected = selectedPermissions.includes("1"); // Admin permission ID is 1

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
                            <Icon icon="pixelarticons:user-plus" className="text-3xl text-primary"/>
                            <span>Create New User</span>
                        </ModalHeader>
                        <ModalBody>
                            <motion.div
                                initial={{opacity: 0, y: 20}}
                                animate={{opacity: 1, y: 0}}
                                transition={{duration: 0.2}}
                                className="flex flex-col gap-4"
                            >
                                {/* Username Input */}
                                <div className="flex flex-col gap-2">
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
                                    <p className="text-xs text-default-500 font-minecraft-body">
                                        A random password will be generated. The user will be required to change it on first login.
                                    </p>
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
                                                value={permission.id.toString()}
                                                classNames={{
                                                    label: "font-minecraft-body",
                                                    wrapper: "rounded-none"
                                                }}
                                                className={permission.name === "Admin" ? "text-primary font-semibold" : ""}
                                            >
                                                <div className="flex flex-col">
                                                    <span>{permission.name}</span>
                                                    <span className="text-xs text-default-500">
                                                        {getPermissionDescription(permission.name)}
                                                    </span>
                                                </div>
                                            </Checkbox>
                                        ))}
                                    </CheckboxGroup>
                                </div>
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
                                startContent={!loading ? <Icon icon="pixelarticons:user-plus"/> : null}
                            >
                                Create User
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
