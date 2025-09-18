import {AnimatePresence, motion} from "framer-motion";
import {addToast, Button, Form, Input, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import Checkbox from "../components/extended/Checkbox.tsx";
import {useState} from "react";
import {PasswordInput} from "../components/extended/PasswordInput.tsx";
import {Tooltip} from "../components/extended/Tooltip.tsx";
import {useAuthentication} from "../providers/AuthenticationProvider.tsx";
import {useHostInfo} from "../providers/HostInfoProvider.tsx";

export default function Signup()
{
    const [unloading, setUnloading] = useState(false);
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [username, setUsername] = useState("");
    const [termsAccepted, setTermsAccepted] = useState(false);
    const {register, login} = useAuthentication();
    const {refreshHostInfo} = useHostInfo();

    // Password validation
    const passwordErrors = [];
    if (password.length > 0 && password.length < 8)
    {
        passwordErrors.push("Password must be at least 8 characters long.");
    }
    if (password.length > 0 && (password.match(/[A-Z]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 uppercase letter.");
    }
    if (password.length > 0 && (password.match(/[a-z]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 lowercase letter.");
    }
    if (password.length > 0 && (password.match(/[0-9]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 number.");
    }
    if (password.length > 0 && (password.match(/[!@#$%^&*()_+\[\]{}|;:,.<>?]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 special character (!@#$%^&*()_+[]{}|;:,.<>?).");
    }

    // Confirm password validation
    const confirmPasswordErrors = [];
    if (confirmPassword.length > 0 && confirmPassword !== password)
    {
        confirmPasswordErrors.push("Passwords do not match.");
    }

    // Username validation
    const usernameErrors = [];
    if (username.length > 0 && username.length < 3)
    {
        usernameErrors.push("Username must be at least 3 characters long.");
    }
    if (username.length > 20)
    {
        usernameErrors.push("Username cannot be longer than 20 characters.");
    }

    // Check if form is valid
    const isFormValid = username.length >= 3 &&
        username.length <= 20 &&
        passwordErrors.length === 0 &&
        password.length > 0 &&
        confirmPasswordErrors.length === 0 &&
        confirmPassword.length > 0 &&
        termsAccepted;

    return (
        <AnimatePresence>
            <div className={"flex flex-col items-center justify-center grow mt-48"}>
                <motion.h1
                    className={"text-7xl text-primary"}
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                    exit={{opacity: 0, y: 50}}
                    transition={{duration: 0.2}}
                >
                    New User
                </motion.h1>
                <motion.h4
                    className={"text-4xl"}
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                    exit={{opacity: 0, y: 50}}
                    transition={{duration: 0.2, delay: 0.1}}
                >
                    Obsidian Server panel
                </motion.h4>
                <Form className={"mt-6 w-4/5 max-w-lg min-w-48"}
                      autoComplete={"off"}
                      onSubmit={async e =>
                      {
                          e.preventDefault();
                          if (unloading || !isFormValid) return;

                          try
                          {
                              await register(username, password);
                              await refreshHostInfo();
                              await login(username, password, false);
                              setUnloading(true);
                              setTimeout(() => setUnloading(false), 1000);
                          } catch (error: any)
                          {
                              addToast({
                                  title: "Registration Failed",
                                  description: error.message || "An error occurred during registration.",
                                  color: "danger"
                              });
                          }
                      }}
                >
                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.2}}
                    >
                        <Input
                            id={"signup-username"}
                            name={"username"}
                            label={"Username"}
                            placeholder={"Choose a username"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"off"}
                            endContent={<Icon icon={"pixelarticons:users"} className={"mr-2"}/>}
                            value={username}
                            onValueChange={setUsername}
                            isInvalid={usernameErrors.length > 0}
                            errorMessage={usernameErrors.length > 0 ? (
                                <ul className={"list-disc list-inside"}>
                                    {usernameErrors.map((error, i) => (
                                        <li key={i}>{error}</li>
                                    ))}
                                </ul>
                            ) : undefined}
                        />
                    </motion.div>
                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.3}}
                    >
                        <PasswordInput
                            id={"signup-password"}
                            name={"password"}
                            label={"Password"}
                            placeholder={"Choose a strong password"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"new-password"}
                            value={password}
                            onValueChange={setPassword}
                            isInvalid={passwordErrors.length > 0}
                            allowPasswordGeneration
                            onPasswordGeneration={password =>
                            {
                                setPassword(password);
                                setConfirmPassword(password);
                                addToast({
                                    title: "Password Generated",
                                    description: "A secure password has been generated and filled in."
                                });
                            }}
                            errorMessage={passwordErrors.length > 0 ? (
                                <ul className={"list-disc list-inside"}>
                                    {passwordErrors.map((error, i) => (
                                        <li key={i}>{error}</li>
                                    ))}
                                </ul>
                            ) : undefined}
                        />
                    </motion.div>
                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.4}}
                    >
                        <PasswordInput
                            id={"signup-confirm-password"}
                            name={"confirmPassword"}
                            label={"Confirm Password"}
                            placeholder={"Re-enter your password"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"new-password"}
                            value={confirmPassword}
                            onValueChange={setConfirmPassword}
                            isInvalid={confirmPasswordErrors.length > 0}
                            errorMessage={confirmPasswordErrors.length > 0 ? (
                                <ul className={"list-disc list-inside"}>
                                    {confirmPasswordErrors.map((error, i) => (
                                        <li key={i}>{error}</li>
                                    ))}
                                </ul>
                            ) : undefined}
                        />
                    </motion.div>

                    <motion.div
                        className={"w-full flex flex-row items-center gap-2"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.5}}
                    >
                        <Checkbox
                            label={"I agree to the Terms of Service"}
                            name={"terms"}
                            labelPlacement={"left"}
                            fullWidth
                            isRequired
                            checked={termsAccepted}
                            onChange={setTermsAccepted}
                        />
                        <Tooltip content={"Read the Terms of Service"} placement={"top"}>
                            <Button
                                as={Link}
                                href={"https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/blob/main/terms-of-service.md"}
                                target={"_blank"}
                                isIconOnly
                                radius={"none"}
                                size={"sm"}
                                className={"text-medium"}
                                variant={"ghost"}
                            >
                                <Icon icon={"pixelarticons:open"}/>
                            </Button>
                        </Tooltip>
                    </motion.div>

                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.6}}
                    >
                        <Button
                            radius={"none"}
                            className={"font-minecraft-body mt-4 w-full"}
                            color={"primary"}
                            type={"submit"}
                            isLoading={unloading}
                            isDisabled={!isFormValid || unloading}
                        >
                            Register
                        </Button>
                    </motion.div>
                </Form>
            </div>
        </AnimatePresence>
    );
}