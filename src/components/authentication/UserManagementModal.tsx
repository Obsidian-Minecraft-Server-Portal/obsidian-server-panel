import {useEffect, useState} from "react";
import {Button, Chip, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Modal, ModalBody, ModalDialog, ModalFooter, ModalHeader, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, useOverlayState} from "@heroui/react";
import {Input} from "../extended/Input.tsx";
import {Icon} from "@iconify-icon/react";
import {AnimatePresence, motion} from "motion/react";
import $ from "jquery";
import {PermissionFlag, UserData} from "../../types/UserTypes.ts";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {MessageOptions, MessageResponseType} from "../MessageModal.tsx";
import CreateUserModal from "./CreateUserModal.tsx";
import EditUserModal from "./EditUserModal.tsx";

interface UserManagementModalProps
{
    isOpen: boolean;
    onClose: () => void;
}

export default function UserManagementModal({isOpen, onClose}: UserManagementModalProps)
{
    const {user: currentUser} = useAuthentication();
    const messageApi = useMessage();
    const [users, setUsers] = useState<UserData[]>([]);
    const [filteredUsers, setFilteredUsers] = useState<UserData[]>([]);
    const [searchQuery, setSearchQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [permissions, setPermissions] = useState<PermissionFlag[]>([]);

    // Modal states
    const {isOpen: isCreateOpen, open: openCreate, close: closeCreate} = useOverlayState();
    const {isOpen: isEditOpen, open: openEdit, close: closeEdit} = useOverlayState();
    const [selectedUser, setSelectedUser] = useState<UserData | null>(null);

    // Load users and permissions on modal open
    useEffect(() =>
    {
        if (isOpen)
        {
            loadUsers();
            loadPermissions();
        }
    }, [isOpen]);

    // Filter users based on search query
    useEffect(() =>
    {
        if (!searchQuery.trim())
        {
            setFilteredUsers(users);
        } else
        {
            const filtered = users.filter(user =>
                user.username.toLowerCase().includes(searchQuery.toLowerCase())
            );
            setFilteredUsers(filtered);
        }
    }, [users, searchQuery]);

    const loadUsers = async () =>
    {
        setLoading(true);
        try
        {
            const response: UserData[] = await $.get("/api/auth/users/");
            setUsers(response || []);
        } catch (error: any)
        {
            console.error("Failed to load users:", error);
            await messageApi.open({
                title: "Error",
                body: "Failed to load users. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally
        {
            setLoading(false);
        }
    };

    const loadPermissions = async () =>
    {
        try
        {
            const response: PermissionFlag[] = await $.get("/api/auth/permissions");
            setPermissions(response);
        } catch (error: any)
        {
            console.error("Failed to load permissions:", error);
        }
    };

    const showMessage = async (options: MessageOptions) =>
    {
        return await messageApi.open(options);
    };

    const handleDeleteUser = async (user: UserData) =>
    {
        const confirmed = await messageApi.open({
            title: "Delete User",
            body: `Are you sure you want to delete the user "${user.username}"? This action cannot be undone.`,
            responseType: MessageResponseType.YesNo,
            severity: "danger",
            icon: "pixelarticons:trash"
        });

        if (confirmed)
        {
            await confirmDeleteUser(user.id);
        }
    };

    const confirmDeleteUser = async (userId: string) =>
    {
        try
        {
            await $.ajax(`/api/auth/users/${userId}`, {method: "DELETE"});
            await messageApi.open({
                title: "Success",
                body: "User deleted successfully.",
                responseType: MessageResponseType.Close,
                severity: "success"
            });
            loadUsers();
        } catch (error: any)
        {
            console.error("Failed to delete user:", error);
            await messageApi.open({
                title: "Error",
                body: "Failed to delete user. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    };

    const handleForcePasswordReset = async (user: UserData) =>
    {
        try
        {
            await $.post(`/api/auth/users/${user.id}/force-password-reset`);
            await messageApi.open({
                title: "Success",
                body: `Password reset forced for user "${user.username}". They will need to change their password on next login.`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });
            loadUsers();
        } catch (error: any)
        {
            console.error("Failed to force password reset:", error);
            await messageApi.open({
                title: "Error",
                body: "Failed to force password reset. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    };

    const handleToggleUserStatus = async (user: UserData) =>
    {
        try
        {
            await $.ajax(`/api/auth/users/${user.id}`, {
                method: "PUT",
                data: JSON.stringify({is_active: !user.is_active}),
                contentType: "application/json"
            });
            await messageApi.open({
                title: "Success",
                body: `User "${user.username}" has been ${user.is_active ? "disabled" : "enabled"}.`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });
            loadUsers();
        } catch (error: any)
        {
            console.error("Failed to toggle user status:", error);
            await messageApi.open({
                title: "Error",
                body: "Failed to update user status. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    };

    const handleEditUser = (user: UserData) =>
    {
        setSelectedUser(user);
        openEdit();
    };

    const formatDate = (dateString: string) =>
    {
        return new Date(dateString).toLocaleDateString();
    };

    const getPermissionNames = (userPermissions: PermissionFlag[]) =>
    {
        return userPermissions.map(p => p.name).join(", ");
    };

    const hasAdminPermission = currentUser?.permissions?.some((p: any) =>
        p.name === "Admin" || p.name === "ManageUsers"
    );

    if (!hasAdminPermission)
    {
        return null;
    }

    return (
        <>
            <Modal
                isOpen={isOpen}
                onOpenChange={(open) => !open && onClose()}
            >
                <ModalDialog>
                    <>
                        <ModalHeader>
                                <Icon icon="pixelarticons:users" className="text-3xl"/>
                                <span>Manage Users</span>
                            </ModalHeader>
                            <ModalBody>
                                <div className="flex flex-col gap-4">
                                    {/* Search and Actions Bar */}
                                    <div className="flex flex-row gap-2 items-center">
                                        <Input
                                            placeholder="Search users..."
                                            value={searchQuery}
                                            onValueChange={setSearchQuery}
                                            startContent={<Icon icon="pixelarticons:search"/>}
                                            className="flex-1 rounded-none"
                                        />
                                        <Button
                                            variant="primary"
                                            className="rounded-none"
                                            onPress={openCreate}
                                        >
                                            <Icon icon="pixelarticons:user-plus"/> Create User
                                        </Button>
                                        <Button
                                            variant="outline"
                                            className="rounded-none"
                                            onPress={loadUsers}
                                            isPending={loading}
                                        >
                                            <Icon icon="pixelarticons:reload"/> Refresh
                                        </Button>
                                    </div>

                                    {/* Users Table */}
                                    <AnimatePresence>
                                        {loading ? (
                                            <div className="flex justify-center items-center h-32">
                                                <Spinner size="lg"/>
                                            </div>
                                        ) : (
                                            <motion.div
                                                initial={{opacity: 0, y: 20}}
                                                animate={{opacity: 1, y: 0}}
                                                exit={{opacity: 0, y: -20}}
                                                transition={{duration: 0.2}}
                                            >
                                                <Table
                                                    className="rounded-none font-minecraft-body"
                                                >
                                                    <TableHeader>
                                                        <TableColumn>USERNAME</TableColumn>
                                                        <TableColumn>PERMISSIONS</TableColumn>
                                                        <TableColumn>STATUS</TableColumn>
                                                        <TableColumn>JOINED</TableColumn>
                                                        <TableColumn>LAST ONLINE</TableColumn>
                                                        <TableColumn>ACTIONS</TableColumn>
                                                    </TableHeader>
                                                    <TableBody items={filteredUsers}>
                                                        {(user) => (
                                                            <TableRow key={user.id}>
                                                                <TableCell>
                                                                    <div className="flex flex-col">
                                                                        <span className="font-semibold">{user.username}</span>
                                                                        {user.needs_password_change && (
                                                                            <Chip size="sm" color="warning" variant="soft">
                                                                                Password reset required
                                                                            </Chip>
                                                                        )}
                                                                    </div>
                                                                </TableCell>
                                                                <TableCell>
                                                                    <span className="text-sm text-default-500">
                                                                        {getPermissionNames(user.permissions)}
                                                                    </span>
                                                                </TableCell>
                                                                <TableCell>
                                                                    <Chip
                                                                        color={user.is_active ? "success" : "danger"}
                                                                        variant="soft"
                                                                    >
                                                                        {user.is_active ? "Active" : "Disabled"}
                                                                    </Chip>
                                                                </TableCell>
                                                                <TableCell>{formatDate(user.join_date)}</TableCell>
                                                                <TableCell>{formatDate(user.last_online)}</TableCell>
                                                                <TableCell>
                                                                    <Dropdown>
                                                                        <DropdownTrigger>
                                                                            <Button
                                                                                isIconOnly
                                                                                variant="ghost"
                                                                            className="rounded-none"
                                                                            >
                                                                                <Icon icon="pixelarticons:more-horizontal"/>
                                                                            </Button>
                                                                        </DropdownTrigger>
                                                                        <DropdownMenu>
                                                                            <DropdownItem
                                                                                key="edit"
                                                                                onPress={() => handleEditUser(user)}
                                                                            >
                                                                                <Icon icon="pixelarticons:edit"/> Edit User
                                                                            </DropdownItem>
                                                                            <DropdownItem
                                                                                key="toggle-status"
                                                                                onPress={() => handleToggleUserStatus(user)}
                                                                            >
                                                                                <Icon icon={user.is_active ? "pixelarticons:power" : "pixelarticons:power-on"}/> {user.is_active ? "Disable" : "Enable"}
                                                                            </DropdownItem>
                                                                            <DropdownItem
                                                                                key="reset-password"
                                                                                onPress={() => handleForcePasswordReset(user)}
                                                                            >
                                                                                <Icon icon="pixelarticons:key"/> Force Password Reset
                                                                            </DropdownItem>
                                                                            <DropdownItem
                                                                                key="delete"
                                                                                className="text-danger"
                                                                                onPress={() => handleDeleteUser(user)}
                                                                            >
                                                                                <Icon icon="pixelarticons:trash"/> Delete User
                                                                            </DropdownItem>
                                                                        </DropdownMenu>
                                                                    </Dropdown>
                                                                </TableCell>
                                                            </TableRow>
                                                        )}
                                                    </TableBody>
                                                </Table>
                                            </motion.div>
                                        )}
                                    </AnimatePresence>
                                </div>
                            </ModalBody>
                            <ModalFooter>
                                <Button onPress={onClose} className="rounded-none">
                                    Close
                                </Button>
                            </ModalFooter>
                    </>
                </ModalDialog>
            </Modal>

            {/* Create User Modal */}
            <CreateUserModal
                isOpen={isCreateOpen}
                onClose={closeCreate}
                permissions={permissions}
                onUserCreated={loadUsers}
                onShowMessage={showMessage}
            />

            {/* Edit User Modal */}
            {selectedUser && (
                <EditUserModal
                    isOpen={isEditOpen}
                    onClose={() =>
                    {
                        closeEdit();
                        setSelectedUser(null);
                    }}
                    user={selectedUser}
                    permissions={permissions}
                    onUserUpdated={loadUsers}
                    onShowMessage={showMessage}
                />
            )}
        </>
    );
}
