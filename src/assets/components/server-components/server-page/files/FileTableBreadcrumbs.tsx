import {BreadcrumbItem, Breadcrumbs} from "@heroui/react";

type FileTableBreadcrumbsProperties = {
    onNavigate: (path: string) => void;
    paths: string[];
}

export function FileTableBreadcrumbs(props: FileTableBreadcrumbsProperties)
{
    const {paths, onNavigate} = props;

    return (
        <Breadcrumbs variant={"solid"} radius={"none"}>
            <BreadcrumbItem
                key="root"
                onPress={() =>
                {
                    onNavigate("");
                }}
            >
                Root
            </BreadcrumbItem>
            {(!paths.length || paths.every(p => p.trim() === "")) ? (
                <></>
            ) : (
                paths.filter(i => i.trim() !== "").map((path, index) => (
                    <BreadcrumbItem
                        key={path}
                        onPress={() =>
                        {
                            if (index >= paths.length)
                                return;
                            let newPath = paths.slice(0, index + 1).join("/");
                            if (!newPath.endsWith("/")) newPath += "/";
                            onNavigate(newPath);
                        }}
                    >
                        {path === "" || path === "/" ? "Root" : path}
                    </BreadcrumbItem>
                ))
            )}
        </Breadcrumbs>
    );
}