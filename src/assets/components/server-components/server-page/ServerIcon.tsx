import {addToast} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useState} from "react";
import {ImageCropModal} from "./options/ImageCropModal.tsx";
import {useServer} from "../../../providers/ServerProvider.tsx";
import {resizeImage} from "../../../ts/image-resizer.ts";

type ServerIconProperties = {
    id: string;
    isChangeEnabled?: boolean;
    size?: "sm" | "md" | "lg" | "xl";
}


const sizeClasses = {
    sm: {size: 64},
    md: {size: 128},
    lg: {size: 256},
    xl: {size: 512}
};

export function ServerIcon(props: ServerIconProperties)
{
    const {id, isChangeEnabled, size} = props;
    const [isEditing, setIsEditing] = useState(false);
    const [image, setImage] = useState<File | null>(null);
    const {uploadFile} = useServer();

    const handleEditClick = async () =>
    {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = "image/*";
        input.multiple = false;
        input.addEventListener("change", async (event) =>
        {
            const files = (event.target as HTMLInputElement).files;
            if (files && files.length === 1)
            {
                const file = files[0];
                setImage(file);
                setIsEditing(true);
            } else
            {
                addToast({
                    title: "Error",
                    description: "Please select a single image file.",
                    color: "danger"
                });
            }
        });

        input.click();
    };

    return (
        <>
            <ImageCropModal
                image={image}
                isOpen={isEditing}
                onClose={async value =>
                {
                    console.log("Cropped image:", value);
                    setIsEditing(false);
                    if (value)
                    {
                        await uploadFile(await resizeImage(value, 64, 64), "");
                    }
                }}
            />
            <div className={"relative"}>
                <img src={`/api/server/${id}/icon?v=${Date.now()}`} alt={"server icon"} width={sizeClasses[size ?? "md"].size} height={sizeClasses[size ?? "md"].size}/>
                {isChangeEnabled && (
                    <div className={"absolute inset-0 w-full h-full bg-black/50 backdrop-blur-sm z-10 flex justify-center items-center font-minecraft-body text-4xl border-1 border-primary opacity-0 hover:opacity-100 transition-all duration-200 cursor-pointer"} onClick={handleEditClick}>
                        <Icon icon={"pixelarticons:edit-box"}/>
                    </div>
                )}
            </div>
        </>
    );
}