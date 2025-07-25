// @ts-ignore
import {Prism as SyntaxHighlighter} from "react-syntax-highlighter";
// @ts-ignore
import {duotoneSea} from "react-syntax-highlighter/dist/esm/styles/prism";
import "../../../css/log-view.css";

type LogViewProps = {
    log: string;
}

export function LogView(props: LogViewProps)
{
    const {log} = props;
    return (
        <>
            <div id={"log-view"} className={"h-full overflow-y-auto bg-background px-4 py-4 pb-14"}>
                <SyntaxHighlighter language={"birb"} style={duotoneSea} showLineNumbers wrapLines={true} wrapLongLines={true} lineProps={{"className": "flex-wrap h-full"}}>
                    {log.trim()}
                </SyntaxHighlighter>
            </div>
        </>
    );
}