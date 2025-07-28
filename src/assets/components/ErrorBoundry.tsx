import React, {Component, ReactNode} from "react";
import {Button} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

interface ErrorBoundaryState
{
    hasError: boolean;
    error?: Error;
}

interface ErrorBoundaryProps
{
    children: ReactNode;
    fallback?: ReactNode;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState>
{
    constructor(props: ErrorBoundaryProps)
    {
        super(props);
        this.state = {hasError: false};
    }

    static getDerivedStateFromError(error: Error): ErrorBoundaryState
    {
        return {hasError: true, error};
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo)
    {
        console.error("Table Error Boundary caught an error:", error, errorInfo);
    }

    render()
    {
        if (this.state.hasError)
        {
            return this.props.fallback || (
                <div className="p-4 text-center text-danger gap-4 flex flex-col items-center justify-center">
                    <h2 className={"text-2xl"}>An unexpected error has occurred!</h2>
                    <Button
                        onPress={() => this.setState({hasError: false})}
                        className={"font-minecraft-body"}
                        radius={"none"}
                        startContent={<Icon className={"text-xl"} icon={"pixelarticons:repeat"}/>}
                        color={"primary"}
                    >
                        Try again
                    </Button>
                </div>
            );
        }

        return this.props.children;
    }
}