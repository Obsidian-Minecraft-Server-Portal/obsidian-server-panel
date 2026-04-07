import {BreadcrumbsItem, Breadcrumbs} from "@heroui/react";

type FileTableBreadcrumbsProperties = {
    onNavigate: (path: string) => void;
    paths: string[];
}

export function FileTableBreadcrumbs(props: FileTableBreadcrumbsProperties)
{
    const {paths, onNavigate} = props;

    return (
        <Breadcrumbs className="rounded-none">
            <BreadcrumbsItem
                key="root"
                onPress={() =>
                {
                    onNavigate("");
                }}
            >
                Root
            </BreadcrumbsItem>
            {(!paths.length || paths.every(p => p.trim() === "")) ? (
                <></>
            ) : (
                paths.filter(i => i.trim() !== "").map((path, index) => (
                    <BreadcrumbsItem
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
                    </BreadcrumbsItem>
                ))
            )}
        </Breadcrumbs>
    );
}