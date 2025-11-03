import {useEffect, useState} from "react";
import {
    Button as HeroButton,
    Card,
    CardBody,
    Chip,
    Divider,
    Dropdown,
    DropdownItem,
    DropdownMenu,
    DropdownTrigger,
    Input,
    Spinner,
    Table,
    TableBody,
    TableCell,
    TableColumn,
    TableHeader,
    TableRow,
    useDisclosure
} from "@heroui/react";
import {Button} from "../../extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import $ from "jquery";
import {PermissionFlag, UserData} from "../../../types/UserTypes.ts";
import {useAuthentication} from "../../../providers/AuthenticationProvider.tsx";
import {MessageOptions, MessageResponseType} from "../../MessageModal.tsx";
import CreateUserModal from "../../authentication/CreateUserModal.tsx";
import EditUserModal from "../../authentication/EditUserModal.tsx";

interface UserSettingsProps {
    onShowMessage: (options: MessageOptions) => Promise<boolean>;
}

export function UserSettings({onShowMessage}: UserSettingsProps) {
    const {user: currentUser} = useAuthentication();
    const [users, setUsers] = useState<UserData[]>([]);
    const [filteredUsers, setFilteredUsers] = useState<UserData[]>([]);
    const [searchQuery, setSearchQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [permissions, setPermissions] = useState<PermissionFlag[]>([]);

    // Modal states
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const {isOpen: isEditOpen, onOpen: onEditOpen, onClose: onEditClose} = useDisclosure();
    const [selectedUser, setSelectedUser] = useState<UserData | null>(null);

    useEffect(() => {
        loadUsers();
        loadPermissions();
    }, []);

    useEffect(() => {
        if (!searchQuery.trim()) {
            setFilteredUsers(users);
        } else {
            const filtered = users.filter(user =>
                user.username.toLowerCase().includes(searchQuery.toLowerCase())
            );
            setFilteredUsers(filtered);
        }
    }, [users, searchQuery]);

    const loadUsers = async () => {
        setLoading(true);
        try {
            const response: UserData[] = await $.get("/api/auth/users/");
            setUsers(response || []);
        } catch (error: any) {
            console.error("Failed to load users:", error);
            onShowMessage({
                title: "Error",
                body: "Failed to load users. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally {
            setLoading(false);
        }
    };

    const loadPermissions = async () => {
        try {
            const response: PermissionFlag[] = await $.get("/api/auth/permissions");
            setPermissions(response);
        } catch (error: any) {
            console.error("Failed to load permissions:", error);
        }
    };

    const handleDeleteUser = async (user: UserData) => {
        const confirmed = await onShowMessage({
            title: "Delete User",
            body: `Are you sure you want to delete the user "${user.username}"? This action cannot be undone.`,
            responseType: MessageResponseType.YesNo,
            severity: "danger",
            icon: "pixelarticons:trash"
        });

        if (!confirmed) return;

        try {
            await $.ajax(`/api/auth/users/${user.id}`, {method: "DELETE"});
            await onShowMessage({
                title: "Success",
                body: "User deleted successfully.",
                responseType: MessageResponseType.Close,
                severity: "success"
            });
            loadUsers();
        } catch (error: any) {
            console.error("Failed to delete user:", error);
            await onShowMessage({
                title: "Error",
                body: "Failed to delete user. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    };

    const handleEditUser = (user: UserData) => {
        setSelectedUser(user);
        onEditOpen();
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString();
    };

    // Wrapper to handle MessageOptions properly for child components
    const onShowMessageSync = (options: MessageOptions) => {
        onShowMessage(options);
    };

    return (
        <div className="flex flex-col gap-6">
            <div>
                <h2 className="text-2xl font-minecraft-header mb-2">User Management</h2>
                <p className="text-sm text-default-500 font-minecraft-body">
                    Manage users, permissions, and access control
                </p>
            </div>

            <Divider/>

            {/* Actions Bar */}
            <div className="flex flex-row justify-between items-center gap-4">
                <Input
                    placeholder="Search users..."
                    value={searchQuery}
                    onValueChange={setSearchQuery}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:search"/>}
                    classNames={{
                        input: "font-minecraft-body"
                    }}
                    className="max-w-md"
                />
                <Button
                    color="primary"
                    onPress={onCreateOpen}
                    startContent={<Icon icon="pixelarticons:user-plus"/>}
                >
                    Create User
                </Button>
            </div>

            {/* Users Table */}
            {loading ? (
                <Card className="bg-default/5">
                    <CardBody className="p-8 flex items-center justify-center">
                        <Spinner size="lg"/>
                        <p className="mt-4 font-minecraft-body">Loading users...</p>
                    </CardBody>
                </Card>
            ) : filteredUsers.length === 0 ? (
                <Card className="bg-default/5">
                    <CardBody className="p-8 text-center">
                        <Icon icon="pixelarticons:users" className="text-4xl mx-auto mb-2 opacity-50"/>
                        <p className="font-minecraft-body opacity-50">
                            {searchQuery ? "No users found matching your search" : "No users found"}
                        </p>
                    </CardBody>
                </Card>
            ) : (
                <Table
                    aria-label="Users table"
                    radius="none"
                    classNames={{
                        wrapper: "rounded-none",
                        th: "font-minecraft-body bg-default-100",
                        td: "font-minecraft-body"
                    }}
                >
                    <TableHeader>
                        <TableColumn>USERNAME</TableColumn>
                        <TableColumn>PERMISSIONS</TableColumn>
                        <TableColumn>CREATED</TableColumn>
                        <TableColumn>STATUS</TableColumn>
                        <TableColumn>ACTIONS</TableColumn>
                    </TableHeader>
                    <TableBody>
                        {filteredUsers.map((user) => (
                            <TableRow key={user.id}>
                                <TableCell>
                                    <div className="flex items-center gap-2">
                                        <Icon icon="pixelarticons:user" className="text-lg"/>
                                        <span className="font-semibold">{user.username}</span>
                                        {user.id === currentUser?.id && (
                                            <Chip size="sm" color="primary" variant="flat">You</Chip>
                                        )}
                                    </div>
                                </TableCell>
                                <TableCell>
                                    <div className="flex flex-wrap gap-1">
                                        {user.permissions && user.permissions.length > 0 ? (
                                            user.permissions.slice(0, 3).map((perm: any) => (
                                                <Chip
                                                    key={perm.id}
                                                    size="sm"
                                                    variant="flat"
                                                    color={perm.name === "Admin" ? "primary" : "default"}
                                                >
                                                    {perm.name}
                                                </Chip>
                                            ))
                                        ) : (
                                            <Chip size="sm" variant="flat">None</Chip>
                                        )}
                                        {user.permissions && user.permissions.length > 3 && (
                                            <Chip size="sm" variant="flat">
                                                +{user.permissions.length - 3}
                                            </Chip>
                                        )}
                                    </div>
                                </TableCell>
                                <TableCell>{formatDate(user.join_date)}</TableCell>
                                <TableCell>
                                    <Chip
                                        size="sm"
                                        color={user.is_active ? "success" : "danger"}
                                        variant="flat"
                                    >
                                        {user.is_active ? "Active" : "Disabled"}
                                    </Chip>
                                </TableCell>
                                <TableCell>
                                    <Dropdown radius="none">
                                        <DropdownTrigger>
                                            <HeroButton
                                                isIconOnly
                                                variant="light"
                                                size="sm"
                                                radius="none"
                                            >
                                                <Icon icon="pixelarticons:more-horizontal"/>
                                            </HeroButton>
                                        </DropdownTrigger>
                                        <DropdownMenu
                                            aria-label="User actions"
                                            className="font-minecraft-body"
                                        >
                                            <DropdownItem
                                                key="edit"
                                                startContent={<Icon icon="pixelarticons:edit"/>}
                                                onPress={() => handleEditUser(user)}
                                            >
                                                Edit User
                                            </DropdownItem>
                                            <DropdownItem
                                                key="delete"
                                                className="text-danger"
                                                color="danger"
                                                startContent={<Icon icon="pixelarticons:trash"/>}
                                                onPress={() => handleDeleteUser(user)}
                                                isDisabled={user.id === currentUser?.id}
                                            >
                                                Delete User
                                            </DropdownItem>
                                        </DropdownMenu>
                                    </Dropdown>
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            )}

            {/* Modals */}
            <CreateUserModal
                isOpen={isCreateOpen}
                onClose={onCreateClose}
                permissions={permissions}
                onUserCreated={loadUsers}
                onShowMessage={onShowMessageSync}
            />

            {selectedUser && (
                <EditUserModal
                    isOpen={isEditOpen}
                    onClose={onEditClose}
                    user={selectedUser}
                    permissions={permissions}
                    onUserUpdated={loadUsers}
                    onShowMessage={onShowMessageSync}
                />
            )}
        </div>
    );
}
