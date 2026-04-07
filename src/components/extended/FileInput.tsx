import {InputHTMLAttributes, useCallback, useState, DragEvent} from "react";
import {Button} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Input} from "./Input.tsx";

type FileInputProps = {
    label?: string;
    description?: string;
    placeholder?: string;
    onChange?: (file: File | File[] | null) => void;
    size?: "sm" | "md" | "lg";
} & Omit<InputHTMLAttributes<HTMLInputElement>, "type">;

export function FileInput(props: FileInputProps)
{
    const [file, setFile] = useState(null as File | File[] | null);
    const [isDragging, setIsDragging] = useState(false);
    const {
        label,
        description,
        placeholder = `Accepts ${props.accept || "any file"}`,
        onChange,
        size: _size = "md"
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
                className={"rounded-none"}
                isReadOnly
                isDisabled
            />
            <Button isIconOnly size={"md"} className={"rounded-none"} onPress={handleUploadClick}><Icon icon={"pixelarticons:cloud-upload"}/></Button>
            {file && (
                <Button isIconOnly size={"md"} className={"rounded-none"} onPress={handleRemoveClick}><Icon icon={"pixelarticons:close"}/></Button>
            )}
        </div>
    );
}
