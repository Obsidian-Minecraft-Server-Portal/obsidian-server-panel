import {addToast, Button, Form, Input, InputOtp, Link} from "@heroui-compat";
import Checkbox from "../components/extended/Checkbox.tsx";
import {Icon} from "@iconify-icon/react";
import {useAuthentication} from "../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "framer-motion";
import {useHostInfo} from "../providers/HostInfoProvider.tsx";
import Signup from "./Signup.tsx";
import {PasswordInput} from "../components/extended/PasswordInput.tsx";
import {useState} from "react";

type LoginStage = "credentials" | "otp" | "forgot" | "reset";

export default function Login()
{
    const {login, verifyLogin, verifyEmail, resendVerification, forgotPassword, resetPassword, isLoggingIn} = useAuthentication();
    const {hostInfo} = useHostInfo();
    const [stage, setStage] = useState<LoginStage>("credentials");
    const [otpPurpose, setOtpPurpose] = useState<"login" | "email">("login");
    const [credentials, setCredentials] = useState({username: "", password: "", remember: false});
    const [code, setCode] = useState("");
    const [isBusy, setIsBusy] = useState(false);

    if (!hostInfo.has_admin_user)
    {
        return <Signup/>;
    }

    const fail = (title: string, error: any) => addToast({
        title,
        description: error.message || "An unexpected error occurred.",
        color: "danger"
    });

    if (stage === "otp")
    {
        return (
            <AnimatePresence>
                <div className={"flex flex-col items-center justify-center grow mt-48"}>
                    <motion.h1 className={"text-5xl text-primary"} initial={{opacity: 0, y: -20}} animate={{opacity: 1, y: 0}} transition={{duration: 0.2}}>
                        {otpPurpose === "login" ? "Two-Factor Auth" : "Verify Email"}
                    </motion.h1>
                    <motion.p
                        className={"text-medium mt-4 font-minecraft-body text-center max-w-lg"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        transition={{duration: 0.2, delay: 0.1}}
                    >
                        A 6-digit code has been sent to your email address. It expires in 10 minutes.
                    </motion.p>
                    <Form className={"mt-6 w-4/5 max-w-lg min-w-48 items-center"}
                          onSubmit={async e =>
                          {
                              e.preventDefault();
                              if (isBusy || code.length !== 6) return;
                              setIsBusy(true);
                              try
                              {
                                  if (otpPurpose === "login") await verifyLogin(credentials.username, credentials.password, code, credentials.remember);
                                  else await verifyEmail(credentials.username, credentials.password, code);
                              } catch (error: any)
                              {
                                  fail("Verification Failed", error);
                                  setCode("");
                              } finally
                              {
                                  setIsBusy(false);
                              }
                          }}
                    >
                        <InputOtp
                            length={6}
                            radius={"none"}
                            className={"font-minecraft-body"}
                            value={code}
                            onValueChange={setCode}
                            autoFocus
                            isRequired
                        />
                        <Button
                            radius={"none"}
                            className={"font-minecraft-body mt-4 w-full"}
                            color={"primary"}
                            type={"submit"}
                            isLoading={isBusy}
                            isDisabled={code.length !== 6}
                        >
                            Verify
                        </Button>
                        <div className={"flex flex-row justify-between w-full mt-2"}>
                            <Link
                                className={"font-minecraft-body text-small cursor-pointer"}
                                onPress={() =>
                                {
                                    setCode("");
                                    setStage("credentials");
                                }}
                            >
                                Back to login
                            </Link>
                            <Link
                                className={"font-minecraft-body text-small cursor-pointer"}
                                onPress={async () =>
                                {
                                    try
                                    {
                                        if (otpPurpose === "email") await resendVerification(credentials.username, credentials.password);
                                        else await login(credentials.username, credentials.password, credentials.remember);
                                        addToast({title: "Code Sent", description: "A new code has been sent to your email address."});
                                    } catch (error: any)
                                    {
                                        fail("Resend Failed", error);
                                    }
                                }}
                            >
                                Resend code
                            </Link>
                        </div>
                    </Form>
                </div>
            </AnimatePresence>
        );
    }

    if (stage === "forgot" || stage === "reset")
    {
        return (
            <AnimatePresence>
                <div className={"flex flex-col items-center justify-center grow mt-48"}>
                    <motion.h1 className={"text-5xl text-primary"} initial={{opacity: 0, y: -20}} animate={{opacity: 1, y: 0}} transition={{duration: 0.2}}>
                        Reset Password
                    </motion.h1>
                    <motion.p
                        className={"text-medium mt-4 font-minecraft-body text-center max-w-lg"}
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        transition={{duration: 0.2, delay: 0.1}}
                    >
                        {stage === "forgot"
                            ? "Enter your username and we'll email a reset code to the address on file."
                            : "Enter the 6-digit code from your email and choose a new password."}
                    </motion.p>
                    <Form className={"mt-6 w-4/5 max-w-lg min-w-48 items-center"}
                          onSubmit={async e =>
                          {
                              e.preventDefault();
                              if (isBusy) return;
                              setIsBusy(true);
                              const data = new FormData(e.currentTarget);
                              try
                              {
                                  if (stage === "forgot")
                                  {
                                      const username = data.get("username") as string;
                                      await forgotPassword(username);
                                      setCredentials(c => ({...c, username}));
                                      setCode("");
                                      setStage("reset");
                                      addToast({title: "Code Sent", description: "If this account has a verified email address, a reset code has been sent."});
                                  } else
                                  {
                                      const password = data.get("password") as string;
                                      await resetPassword(credentials.username, code, password);
                                      setCode("");
                                      setStage("credentials");
                                      addToast({title: "Password Reset", description: "You can now log in with your new password."});
                                  }
                              } catch (error: any)
                              {
                                  fail("Password Reset Failed", error);
                              } finally
                              {
                                  setIsBusy(false);
                              }
                          }}
                    >
                        {stage === "forgot" ? (
                            <Input
                                name={"username"}
                                label={"Username"}
                                placeholder={"Enter your username"}
                                radius={"none"}
                                className={"font-minecraft-body"}
                                isRequired
                                autoComplete={"username"}
                                endContent={<Icon icon={"pixelarticons:users"} className={"mr-2"}/>}
                            />
                        ) : (
                            <>
                                <InputOtp
                                    length={6}
                                    radius={"none"}
                                    className={"font-minecraft-body"}
                                    value={code}
                                    onValueChange={setCode}
                                    autoFocus
                                    isRequired
                                />
                                <PasswordInput
                                    name={"password"}
                                    label={"New Password"}
                                    placeholder={"Choose a new password"}
                                    radius={"none"}
                                    className={"font-minecraft-body"}
                                    isRequired
                                    autoComplete={"new-password"}
                                />
                            </>
                        )}
                        <Button
                            radius={"none"}
                            className={"font-minecraft-body mt-4 w-full"}
                            color={"primary"}
                            type={"submit"}
                            isLoading={isBusy}
                        >
                            {stage === "forgot" ? "Send Reset Code" : "Reset Password"}
                        </Button>
                        <Link
                            className={"font-minecraft-body text-small cursor-pointer mt-2"}
                            onPress={() =>
                            {
                                setCode("");
                                setStage("credentials");
                            }}
                        >
                            Back to login
                        </Link>
                    </Form>
                </div>
            </AnimatePresence>
        );
    }

    return (
        <AnimatePresence>
            <div className={"flex flex-col items-center justify-center grow mt-48"}>
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
                              const result = await login(username, password, rememberMe, 500);
                              if (result !== "success")
                              {
                                  setCredentials({username, password, remember: rememberMe});
                                  setOtpPurpose(result === "requires_2fa" ? "login" : "email");
                                  setCode("");
                                  setStage("otp");
                              }
                          } catch (error: any)
                          {
                              fail("Login Failed", error);
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
                    {hostInfo.smtp_enabled && (
                        <motion.div
                            className={"w-full flex justify-end"}
                            initial={{opacity: 0, y: -20}}
                            animate={{opacity: isLoggingIn ? 0 : 1, y: isLoggingIn ? -50 : 0}}
                            exit={{opacity: 0, y: 50}}
                            transition={{duration: 0.2, delay: 0.6}}
                        >
                            <Link
                                className={"font-minecraft-body text-small cursor-pointer mt-2"}
                                onPress={() => setStage("forgot")}
                            >
                                Forgot password?
                            </Link>
                        </motion.div>
                    )}
                </Form>
            </div>
        </AnimatePresence>
    );
}
