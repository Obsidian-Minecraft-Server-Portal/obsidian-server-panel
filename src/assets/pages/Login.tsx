import {addToast, Button, Form, Input} from "@heroui/react";
import Checkbox from "../components/extended/Checkbox.tsx";
import {Icon} from "@iconify-icon/react";
import {useAuthentication} from "../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "framer-motion";
import {useHostInfo} from "../providers/ServerInfoProvider.tsx";
import Signup from "./Signup.tsx";
import {PasswordInput} from "../components/extended/PasswordInput.tsx";

export default function Login()
{
    const {login, isLoggingIn} = useAuthentication();
    const {hostInfo} = useHostInfo();

    if (!hostInfo.has_admin_user)
    {
        return <Signup/>;
    }


    return (
        <AnimatePresence>
            <div className={"flex flex-col items-center justify-center grow"}>
                <motion.h1
                    className={"text-7xl text-primary"}
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                    exit={{opacity: 0, y: 50}}
                    transition={{duration: 0.2}}
                >
                    Obsidian
                </motion.h1>
                <motion.h4
                    className={"text-4xl"}
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                    exit={{opacity: 0, y: 50}}
                    transition={{duration: 0.2, delay: 0.1}}
                >
                    Server panel
                </motion.h4>
                <Form className={"mt-6 w-4/5 max-w-lg min-w-48"}
                      onSubmit={async e =>
                      {
                          e.preventDefault();
                          if (isLoggingIn) return;

                          const data = new FormData(e.currentTarget);
                          const username = data.get("username") as string;
                          const password = data.get("password") as string;
                          const rememberMe = data.get("remeberme") === "on";
                          try
                          {
                              await login(username, password, rememberMe, 500);
                          } catch (error: any)
                          {
                              addToast({
                                  title: "Login Failed",
                                  description: error.message || "An error occurred during login.",
                                  color: "danger"
                              });
                          }
                      }}
                >
                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.2}}
                    >
                        <Input
                            id={"login-username"}
                            name={"username"}
                            label={"Username"}
                            placeholder={"Enter your username"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"username webauthn"}
                            endContent={<Icon icon={"pixelarticons:users"} className={"mr-2"}/>}
                            errorMessage={"Please provide a username or email address."}
                        />
                    </motion.div>
                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.3}}
                    >
                        <PasswordInput
                            id={"login-password"}
                            name={"password"}
                            label={"Password"}
                            placeholder={"*********"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"current-password webauthn"}
                            errorMessage={"Please provide a password."}
                        />
                    </motion.div>

                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.4}}
                    >
                        <Checkbox label={"Remember Me?"} name={"remeberme"} labelPlacement={"left"} fullWidth/>
                    </motion.div>

                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.5}}
                    >
                        <Button
                            radius={"none"}
                            className={"font-minecraft-body mt-4 w-full"}
                            color={"primary"}
                            type={"submit"}
                            isLoading={isLoggingIn}
                        >
                            Login
                        </Button>
                    </motion.div>
                </Form>
            </div>
        </AnimatePresence>
    );
}