import {Card, CardContent, Separator} from "@heroui/react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import {ModDetails} from "../../types/ModTypes";

interface ModDescriptionProps
{
    modDetails: ModDetails;
}

export function ModDescription({modDetails}: ModDescriptionProps)
{
    // Custom markdown components for proper styling
    const markdownComponents = {
        h1: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h1 className="text-4xl mb-2">{children}</h1>
                <Separator/>
            </div>
        ),
        h2: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h2 className="text-3xl mb-2">{children}</h2>
                <Separator/>
            </div>
        ),
        h3: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h3 className="text-2xl mb-2">{children}</h3>
                <Separator/>
            </div>
        ),
        h4: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h4 className="text-xl mb-2">{children}</h4>
                <Separator/>
            </div>
        ),
        ul: ({children}: any) => (
            <ul className="list-disc ml-8 my-4" style={{listStyleType: "disc"}}>
                {children}
            </ul>
        ),
        img: ({src, alt}: any) => (
            <img
                src={src}
                alt={alt || ""}
                className="my-4"
                loading="lazy"
            />
        )
    };

    return (
        <Card>
            <CardContent className="p-6 min-h-[calc(100dvh_-_440px)] rounded-none">
                {modDetails.body ? (
                    <div className="prose prose-sm max-w-none">
                        <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            rehypePlugins={[rehypeRaw, rehypeSanitize]}
                            components={markdownComponents}
                        >
                            {modDetails.body}
                        </ReactMarkdown>
                    </div>
                ) : (
                    <p className="text-default-600">{modDetails.description}</p>
                )}
            </CardContent>
        </Card>
    );
}
