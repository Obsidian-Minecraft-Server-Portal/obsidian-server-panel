// @ts-ignore
import {Prism as SyntaxHighlighter} from "react-syntax-highlighter";
// @ts-ignore
import {duotoneSea} from "react-syntax-highlighter/dist/esm/styles/prism";
import "../../../../css/log-view.css";
import {forwardRef} from "react";

type LogViewProps = {
    log: string[];
}

export const LogView = forwardRef<HTMLDivElement, LogViewProps>((props, ref) =>
{
    const {log} = props;
    return (
        <>
            <div ref={ref} id={"log-view"} className={"h-full overflow-y-auto bg-background px-4 py-4 pb-14"}>
                <SyntaxHighlighter language={"birb"} style={duotoneSea} showLineNumbers wrapLines={true} wrapLongLines={true} lineProps={{"className": "flex-wrap h-full"}}>
                    {log.join("\n").trim()}
                </SyntaxHighlighter>
            </div>
        </>
    );
});