import {forwardRef, InputHTMLAttributes, useCallback, useState, DragEvent} from "react";
import {Button, Input} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

type FileInputProps = {
    label?: string;
    description?: string;
    placeholder?: string;
    onChange?: (file: File | File[] | null) => void;
    size?: "sm" | "md" | "lg";
} & Omit<InputHTMLAttributes<HTMLInputElement>, "type">;

export const FileInput = forwardRef<HTMLInputElement, FileInputProps>((props) =>
{
    const [file, setFile] = useState(null as File | File[] | null);
    const [isDragging, setIsDragging] = useState(false);
    const {
        label,
        description,
        placeholder = `Accepts ${props.accept || "any file"}`,
        onChange,
        size = "md"
    } = props;

    const handleUploadClick = () =>
    {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = props.accept || "*/*";
        input.addEventListener("change", event =>
        {
            const files = (event.target as HTMLInputElement).files;
            if (files && files.length > 0)
            {
                let selectedFile: File | File[] = files.length === 1 ? files[0] : Array.from(files);
                setFile(selectedFile);
                onChange?.(selectedFile);
            }
        });
        input.click();
    };

    const handleRemoveClick = useCallback(() =>
    {
        setFile(null);
        onChange?.(null);
    }, [file]);

    const handleDrop = (e: DragEvent) =>
    {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);

        const files = e.dataTransfer.files;
        if (files && files.length > 0)
        {
            let selectedFile: File | File[] = files.length === 1 ? files[0] : Array.from(files);
            setFile(selectedFile);
            onChange?.(selectedFile);
        }
    };

    const handleDragOver = (e: DragEvent) =>
    {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(true);
    };

    const handleDragLeave = (e: DragEvent) =>
    {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);
    };

    return (
        <div
            className={`w-full flex flex-row gap-2 font-minecraft-body transition-background duration-200 ${isDragging ? "ring-2 ring-primary bg-primary/10" : ""}`}
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
        >
            <Input
                label={label}
                placeholder={placeholder}
                description={description}
                value={file ? (Array.isArray(file) ? file.map(f => f.name).join(", ") : file.name) : ""}
                radius={"none"}
                size={size}
                isReadOnly
                isDisabled
                classNames={{
                    base: "data-[disabled]:opacity-100"
                }}
            />
            <Button isIconOnly size={"md"} radius={"none"} onPress={handleUploadClick}><Icon icon={"pixelarticons:cloud-upload"}/></Button>
            {file && (
                <Button isIconOnly size={"md"} radius={"none"} onPress={handleRemoveClick}><Icon icon={"pixelarticons:close"}/></Button>
            )}
        </div>
    );
});