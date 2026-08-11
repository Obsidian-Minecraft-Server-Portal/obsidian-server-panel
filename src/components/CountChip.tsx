import {Chip} from "@heroui-compat";

/** Count badge that renders nothing at zero. */
export function CountChip({count, className}: { count: number; className?: string })
{
    if (count <= 0) return null;
    return <Chip radius={"full"} size={"sm"} className={`text-tiny font-minecraft-body ${className ?? ""}`}>{count}</Chip>;
}
