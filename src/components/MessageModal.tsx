import {Button, Modal, ModalBody, ModalDialog, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {ReactNode, useEffect} from "react";

export type MessageOptions = {
    title: string;
    body: ReactNode | string;
    responseType: MessageResponseType;
    severity?: "info" | "warning" | "danger" | "success";
    icon?: ReactNode | string;
}

type MessageProperties = {
    isOpen: boolean;
    onClose: (response: boolean | null) => void;
} & MessageOptions;

export enum MessageResponseType
{
    YesNo,
    OkayCancel,
    Close,
}

export default function MessageModal(props: MessageProperties)
{
    const {
        title,
        body,
        responseType,
        icon,
        severity,
        isOpen,
        onClose
    } = props;

    // Handle keyboard events for Enter and Space
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                event.stopPropagation();
                // Trigger the primary action (Yes/Okay/Close)
                onClose(true);
            }
        };

        // Add event listener when modal is open
        document.addEventListener('keydown', handleKeyDown);

        // Cleanup event listener when modal closes or component unmounts
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isOpen, onClose]);

    return (
        <Modal
            isOpen={isOpen}
            onOpenChange={(open) => !open && onClose(responseType === MessageResponseType.Close ? null : false)}
            data-severity={severity}
        >
            <ModalDialog>
                <>
                    <ModalHeader>
                            <span className={"text-3xl h-[30px]"}>{typeof icon === "string" ? <Icon icon={icon}/> : icon == null ? <MessageIcon severity={severity}/> : icon}</span>
                            <span
                                className={"data-[severity=danger]:text-danger data-[severity=warning]:text-warning data-[severity=info]:text-blue-500 data-[severity=success]:text-success"}
                                data-severity={severity}
                            >
                                {title}
                            </span>
                        </ModalHeader>
                        <ModalBody>
                            {body}
                        </ModalBody>
                        <ModalFooter>
                            {({
                                [MessageResponseType.YesNo]: (
                                    <>
                                        <Button className="rounded-none" onPress={() => onClose(true)} variant={severity === "danger" ? "danger" : "primary"} autoFocus>Yes</Button>
                                        <Button className="rounded-none" onPress={() => onClose(false)} variant={"outline"}>No</Button>
                                    </>
                                ),
                                [MessageResponseType.OkayCancel]: (
                                    <>
                                        <Button className="rounded-none" onPress={() => onClose(true)} variant={severity === "danger" ? "danger" : "primary"} autoFocus>Okay</Button>
                                        <Button className="rounded-none" onPress={() => onClose(false)} variant={"outline"}>Cancel</Button>
                                    </>
                                ),
                                [MessageResponseType.Close]: (
                                    <Button className="rounded-none" onPress={() => onClose(true)} autoFocus>Close</Button>
                                )
                            })[responseType]}
                    </ModalFooter>
                </>
            </ModalDialog>
        </Modal>
    );
}

const MessageIcon = ({severity}: { severity: "info" | "warning" | "danger" | "success" | undefined }) =>
{
    switch (severity)
    {
        case "info":
            return <Icon icon={"pixelarticons:info-box"} className={"text-blue-500"}/>;
        case "warning":
            return <Icon icon={"pixelarticons:warning-box"} className={"text-warning"}/>;
        case "danger":
            return <Icon icon={"pixelarticons:alert"} className={"text-danger"}/>;
        case "success":
            return <Icon icon={"pixelarticons:radio-on"} className={"text-success"}/>;
        default:
            return null;
    }
};