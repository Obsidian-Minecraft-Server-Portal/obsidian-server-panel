import {Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Button} from "../extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {PasswordInput} from "../extended/PasswordInput.tsx";
import {useState} from "react";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import $ from "jquery";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../MessageModal.tsx";

type ChangePasswordProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function ChangePasswordModal(props: ChangePasswordProperties)
{
    const {logout} = useAuthentication();
    const [newPassword, setNewPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const messageApi = useMessage();

    // Password validation
    const passwordErrors: string[] = [];
    if (newPassword.length > 0 && newPassword.length < 8)
    {
        passwordErrors.push("Password must be at least 8 characters long.");
    }
    if (newPassword.length > 0 && (newPassword.match(/[A-Z]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 uppercase letter.");
    }
    if (newPassword.length > 0 && (newPassword.match(/[a-z]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 lowercase letter.");
    }
    if (newPassword.length > 0 && (newPassword.match(/[0-9]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 number.");
    }
    if (newPassword.length > 0 && (newPassword.match(/[!@#$%^&*()_+\[\]{}|;:,.<>?]/g) || []).length < 1)
    {
        passwordErrors.push("Password must include at least 1 special character (!@#$%^&*()_+[]{}|;:,.<>?).");
    }

    // Confirm password validation
    const confirmPasswordErrors: string[] = [];
    if (confirmPassword.length > 0 && confirmPassword !== newPassword)
    {
        confirmPasswordErrors.push("Passwords do not match.");
    }

    // Check if form is valid
    const isFormValid = passwordErrors.length === 0 &&
        newPassword.length > 0 &&
        confirmPasswordErrors.length === 0 &&
        confirmPassword.length > 0;

    return (
        <Modal isOpen={props.isOpen} onClose={props.onClose}
               size="2xl"
               scrollBehavior="inside"
               backdrop="blur"
               radius="none"
               closeButton={<Icon icon="pixelarticons:close-box" width={24}/>}
               classNames={{
                   closeButton: "rounded-none"
               }}
               isDismissable={false}
               isKeyboardDismissDisabled={false}
               hideCloseButton
        >
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader className={"font-minecraft-body text-2xl font-normal"}>Change Password</ModalHeader>
                        <ModalBody>
                            <PasswordInput
                                label={"New Password"}
                                autoComplete={"new-password webauthn"}
                                value={newPassword}
                                onValueChange={setNewPassword}
                                allowPasswordGeneration
                                tabIndex={0}
                                autoFocus
                                onPasswordGeneration={value =>
                                {
                                    setNewPassword(value);
                                    setConfirmPassword(value);
                                }}
                                isInvalid={passwordErrors.length > 0}
                                errorMessage={passwordErrors.length > 0 ? (
                                    <ul className={"list-disc list-inside"}>
                                        {passwordErrors.map((error, i) => (
                                            <li key={i}>{error}</li>
                                        ))}
                                    </ul>
                                ) : undefined}
                            />
                            <PasswordInput
                                label={"Confirm Password"}
                                autoComplete={"new-password webauthn"}
                                tabIndex={1}
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
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={() =>
                            {
                                logout();
                                onClose();
                            }}>Logout</Button>
                            <Button
                                color="primary"
                                isDisabled={!isFormValid}
                                onPress={async () =>
                                {
                                    console.log("Password change submitted");
                                    try
                                    {
                                        await $.ajax("/api/auth/change-password", {method: "POST", data: newPassword});
                                        setNewPassword("");
                                        setConfirmPassword("");
                                        onClose();
                                    } catch (e)
                                    {
                                        await messageApi.open({
                                            title: "Error",
                                            body: "Failed to change password. Please try again.",
                                            severity: "danger",
                                            responseType: MessageResponseType.Close
                                        });
                                    }
                                }}
                            >
                                Change Password
                            </Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}