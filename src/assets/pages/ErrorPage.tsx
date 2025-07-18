import {Button, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

export default function ErrorPage()
{
    return (
        <>
            <div className="flex flex-col items-center justify-center grow">
                <h1 className="text-9xl font-minecraft-header text-primary mb-4">404</h1>
                <h4 className="text-4xl font-minecraft-header mb-8">Page Not Found</h4>
                <p className="font-minecraft-body text-center text-foreground/80 mb-8">
                    The page you're looking for doesn't exist or has been moved.
                </p>
                <div className="flex gap-4">
                    <Button radius={"none"} color={"primary"} as={Link} href={"/app"} className="font-minecraft-body" startContent={<Icon icon={"pixel:home-solid"}/>}>
                        Go Home
                    </Button>

                    <Button radius={"none"} as={Link} href={"https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/issues/new"} target={"_blank"} className="font-minecraft-body" startContent={<Icon icon={"pixelarticons:debug"}/>}>
                        Report Issue
                    </Button>
                </div>
            </div>
        </>
    );
}