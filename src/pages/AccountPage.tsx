import {ReactNode, useState} from "react";
import {AnimatePresence, motion} from "framer-motion";
import {Card, CardBody, Chip, Divider} from "@heroui-compat";
import {Icon} from "@iconify-icon/react";
import {Button} from "../components/extended/Button.tsx";
import {useAuthentication} from "../providers/AuthenticationProvider.tsx";
import ChangePasswordModal from "../components/authentication/ChangePasswordModal.tsx";

const formatDate = (value: Date | string | null | undefined): string =>
{
    if (!value) return "Unknown";
    const date = new Date(value);
    return isNaN(date.getTime()) ? "Unknown" : date.toLocaleString();
};

function Field({label, icon, children}: { label: string; icon: string; children: ReactNode })
{
    return (
        <div className={"flex flex-row items-start gap-3 min-w-0"}>
            <Icon icon={icon} className={"text-xl text-primary mt-1 shrink-0"}/>
            <div className={"flex flex-col min-w-0"}>
                <span className={"text-xs text-default-500 font-minecraft-body"}>{label}</span>
                <div className={"font-minecraft-body break-words"}>{children}</div>
            </div>
        </div>
    );
}

export default function AccountPage()
{
    const {user, logout} = useAuthentication();
    const [isChangingPassword, setIsChangingPassword] = useState(false);

    if (!user) return null;

    return (
        <AnimatePresence>
            <motion.div
                className={"flex flex-col gap-4 px-8 py-6 max-w-4xl w-full mx-auto"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: 1, y: 0}}
                transition={{duration: 0.3}}
            >
                <div className={"flex flex-row items-center gap-3 flex-wrap"}>
                    <Icon icon={"pixelarticons:user"} className={"text-4xl text-primary"}/>
                    <h1 className={"text-3xl font-minecraft-header"}>{user.username}</h1>
                    {!user.is_active && <Chip size={"sm"} color={"danger"} variant={"flat"}>Inactive</Chip>}
                </div>

                <Card radius={"none"} className={"shadow-none"}>
                    <CardBody className={"flex flex-col gap-4 p-6"}>
                        <h2 className={"text-xl font-minecraft-header"}>Profile</h2>
                        <Divider/>
                        <div className={"grid grid-cols-1 sm:grid-cols-2 gap-4"}>
                            <Field label={"Username"} icon={"pixelarticons:user"}>{user.username}</Field>
                            <Field label={"Email"} icon={"pixelarticons:mail"}>
                                <div className={"flex flex-row items-center gap-2 flex-wrap"}>
                                    <span>{user.email ?? "Not set"}</span>
                                    {user.email && (
                                        <Chip size={"sm"} variant={"flat"} color={user.email_verified ? "success" : "warning"}>
                                            {user.email_verified ? "Verified" : "Unverified"}
                                        </Chip>
                                    )}
                                </div>
                            </Field>
                            <Field label={"Joined"} icon={"pixelarticons:calendar"}>{formatDate(user.join_date)}</Field>
                            <Field label={"Last online"} icon={"pixelarticons:clock"}>{formatDate(user.last_online)}</Field>
                        </div>
                    </CardBody>
                </Card>

                <Card radius={"none"} className={"shadow-none"}>
                    <CardBody className={"flex flex-col gap-4 p-6"}>
                        <h2 className={"text-xl font-minecraft-header"}>Permissions</h2>
                        <Divider/>
                        {user.permissions?.length ? (
                            <div className={"flex flex-row flex-wrap gap-2"}>
                                {user.permissions.map(permission => (
                                    <Chip
                                        key={permission.id}
                                        size={"sm"}
                                        variant={"flat"}
                                        color={permission.name === "Admin" ? "primary" : "default"}
                                    >
                                        {permission.name}
                                    </Chip>
                                ))}
                            </div>
                        ) : (
                            <p className={"text-default-500 font-minecraft-body"}>No permissions assigned.</p>
                        )}
                    </CardBody>
                </Card>

                <Card radius={"none"} className={"shadow-none"}>
                    <CardBody className={"flex flex-col gap-4 p-6"}>
                        <h2 className={"text-xl font-minecraft-header"}>Security</h2>
                        <Divider/>
                        <div className={"flex flex-row flex-wrap gap-2"}>
                            <Button
                                color={"primary"}
                                startContent={<Icon icon={"pixelarticons:lock"}/>}
                                onPress={() => setIsChangingPassword(true)}
                            >
                                Change Password
                            </Button>
                            <Button
                                color={"danger"}
                                variant={"light"}
                                startContent={<Icon icon={"pixelarticons:logout"}/>}
                                onPress={logout}
                            >
                                Log Out
                            </Button>
                        </div>
                    </CardBody>
                </Card>
            </motion.div>

            <ChangePasswordModal isOpen={isChangingPassword} onClose={() => setIsChangingPassword(false)}/>
        </AnimatePresence>
    );
}
