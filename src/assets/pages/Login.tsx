import {addToast, Button, Form, Input, Tooltip} from "@heroui/react";
import Checkbox from "../components/Checkbox.tsx";
import {Icon} from "@iconify-icon/react";
import {useState} from "react";
import {useAuthentication} from "../providers/AuthenticationProvider.tsx";

export default function Login()
{
    const [showPassword, setShowPassword] = useState(false);
    const {login, isLoggingIn} = useAuthentication();
    return (
        <div className={"flex flex-col items-center justify-center h-screen"}>
            <h1 className={"text-7xl text-primary"}>Obsidian</h1>
            <h4 className={"text-4xl"}>Server panel</h4>
            <Form className={"mt-6 w-4/5 max-w-lg min-w-48"}
                  onSubmit={async e =>
                  {
                      e.preventDefault();
                      if (isLoggingIn) return;

                      // Handle form submission logic here
                      const data = new FormData(e.currentTarget);
                      console.log("Submitted Login Data: ", data);
                      const username = data.get("username") as string;
                      const password = data.get("password") as string;
                      const rememberMe = data.get("remeberme") === "on";
                      try
                      {
                          await login(username, password, rememberMe);
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
                <Input
                    id={"login-password"}
                    name={"password"}
                    label={"Password"}
                    placeholder={"*********"}
                    radius={"none"}
                    type={showPassword ? "text" : "password"}
                    className={"font-minecraft-body"}
                    isRequired
                    autoComplete={"current-password webauthn"}
                    errorMessage={"Please provide a password."}
                    endContent={
                        <Tooltip content={"Toggle Password Visibility"} placement={"top"} radius={"none"} className={"font-minecraft-body"}>
                            <Button isIconOnly size={"sm"} variant={"light"} onPress={() => setShowPassword(prev => !prev)}>
                                <Icon icon={showPassword ? "pixelarticons:eye-closed" : "pixelarticons:eye"} width={16}/>
                            </Button>
                        </Tooltip>
                    }
                />
                <Checkbox label={"Remember Me?"} name={"remeberme"} labelPlacement={"left"} fullWidth/>
                <Button
                    radius={"none"}
                    className={"font-minecraft-body mt-4 w-full"}
                    color={"primary"}
                    type={"submit"}
                    isLoading={isLoggingIn}
                >
                    Login
                </Button>

            </Form>
        </div>
    );
}