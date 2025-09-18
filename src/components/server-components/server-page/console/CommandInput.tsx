import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useState} from "react";
import {Button, Input} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";

export function CommandInput()
{
    const {sendCommand} = useServer();
    const [value, setValue] = useState("");

    const handleCommandSubmit = async () =>
    {
        if (value.trim() === "") return;
        try
        {
            await sendCommand(value.trim());
            setValue(""); // Clear the input after sending the command
        } catch (error)
        {
            console.error("Failed to send command:", error);
        }
    };

    return (

        <div className={"absolute bottom-8 left-8 right-8 font-minecraft-body"}>
            <Input
                placeholder={"Send a command..."}
                radius={"none"}
                value={value}
                onValueChange={setValue}
                startContent={<Icon icon={"mdi:console"}/>}
                endContent={
                    <Tooltip content={"Send Command"}>
                        <Button isIconOnly variant={"light"} size={"sm"} radius={"none"} onPress={handleCommandSubmit}><Icon icon={"mdi:send"}/></Button>
                    </Tooltip>
                }
                onKeyUp={async (e) =>
                {
                    if (e.key === "Enter" && e.currentTarget.value.trim() !== "") await handleCommandSubmit();
                }}
            />
        </div>
    );
}