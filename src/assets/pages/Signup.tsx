import {AnimatePresence, motion} from "framer-motion";
import {addToast, Button, Form, Input} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import Checkbox from "../components/extended/Checkbox.tsx";
import {useState} from "react";
import {PasswordInput} from "../components/extended/PasswordInput.tsx";

export default function Signup()
{
    const [unloading, setUnloading] = useState(false);
    return (
        <AnimatePresence>
            <div className={"flex flex-col items-center justify-center grow"}>
                <motion.h1
                    className={"text-7xl text-primary"}
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                    exit={{opacity: 0, y: 50}}
                    transition={{duration: 0.2}}
                >
                    Registration
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
                      onSubmit={async e =>
                      {
                          e.preventDefault();
                          if (unloading) return;

                          const data = new FormData(e.currentTarget);
                          const username = data.get("username") as string;
                          const password = data.get("password") as string;
                          const rememberMe = data.get("remeberme") === "on";
                          try
                          {
                              console.log("Signup data:", {username, password, rememberMe});
                              // TOOD: Implement signup logic here
                              setUnloading(true);
                              setTimeout(() => setUnloading(false), 1000); // Simulate a delay for the signup process
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
                            autoComplete={"username"}
                            endContent={<Icon icon={"pixelarticons:users"} className={"mr-2"}/>}
                            errorMessage={"Please choose a username."}
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
                            errorMessage={"Please provide a password."}
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
                            id={"signup-confirm-password"}
                            name={"confirmPassword"}
                            label={"Confirm Password"}
                            placeholder={"Re-enter your password"}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            isRequired
                            autoComplete={"new-password"}
                            errorMessage={"Passwords must match."}
                        />
                    </motion.div>

                    <motion.div
                        className={"w-full flex flex-row"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.4}}
                    >
                        <Checkbox label={"I agree to the Terms of Service"} name={"terms"} labelPlacement={"left"} fullWidth isRequired/>
                        <Button isIconOnly radius={"none"}><Icon icon={"pixelarticons:open"} /></Button>
                    </motion.div>

                    <motion.div
                        className={"w-full"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: unloading ? 0 : 1, y: unloading ? -50 : 0}}
                        exit={{opacity: 0, y: 50}}
                        transition={{duration: 0.2, delay: 0.5}}
                    >
                        <Button
                            radius={"none"}
                            className={"font-minecraft-body mt-4 w-full"}
                            color={"primary"}
                            type={"submit"}
                            isLoading={unloading}
                        >
                            Register
                        </Button>
                    </motion.div>
                </Form>
            </div>
        </AnimatePresence>
    );
}