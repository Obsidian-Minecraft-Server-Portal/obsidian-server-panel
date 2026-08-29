import {Button, Input} from "@heroui-compat";
import {Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui-compat";
import {Icon} from "@iconify-icon/react";
import {KeyboardEvent, useEffect, useState} from "react";

export interface ArchiveCreationModalProps
{
    isOpen: boolean;
    defaultName: string;
    entryCount: number;
    onCreate: (name: string) => void;
    onCancel: () => void;
}

export function ArchiveCreationModal({isOpen, defaultName, entryCount, onCreate, onCancel}: ArchiveCreationModalProps)
{
    const [name, setName] = useState(defaultName);

    useEffect(() =>
    {
        if (isOpen) setName(defaultName);
    }, [isOpen, defaultName]);

    const submit = () =>
    {
        if (name.trim() === "") return;
        onCreate(name);
    };

    const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) =>
    {
        if (e.key === "Enter")
        {
            e.preventDefault();
            submit();
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onCancel}
            size={"sm"}
            radius={"none"}
            isDismissable={true}
        >
            <ModalContent>
                <ModalHeader className={"flex flex-row items-center gap-2 text-2xl"}>
                    <Icon icon={"pixelarticons:archive"} className={"h-[30px] text-3xl"}/>
                    <span>Create Archive</span>
                </ModalHeader>
                <ModalBody>
                    <p className={"font-minecraft-body text-default-500"}>
                        Archive {entryCount} {entryCount === 1 ? "entry" : "entries"} into a zip file.
                    </p>
                    <Input
                        aria-label={"Archive name"}
                        label={"Archive name"}
                        value={name}
                        onValueChange={setName}
                        onKeyDown={handleKeyDown}
                        autoFocus
                        radius={"none"}
                        placeholder={"archive-name"}
                    />
                </ModalBody>
                <ModalFooter>
                    <Button radius={"none"} variant={"ghost"} onPress={onCancel}>Cancel</Button>
                    <Button radius={"none"} color={"primary"} isDisabled={name.trim() === ""} onPress={submit}>Create</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}
