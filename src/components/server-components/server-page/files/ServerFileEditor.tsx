import {cn} from "@heroui/react";
import {Editor} from "@monaco-editor/react";
import {AnimatePresence, motion} from "framer-motion";
import {forwardRef, useCallback, useImperativeHandle, useRef} from "react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {getMonacoLanguage, isTextFile} from "../../../../ts/file-type-match.ts";
import {registerMinecraftPropertiesLanguage} from "../../../../ts/minecraft-properties-language.ts";

// Define the theme outside the component
const defineObsidianTheme = (monaco: any) =>
{
    monaco.editor.defineTheme("obsidian-editor-theme", {
        base: "vs-dark",
        inherit: true,
        rules: [
            {token: "key", foreground: "#47ebb4"},
            {token: "value", foreground: "#CE9178"},
            {token: "comment", foreground: "#57718e", fontStyle: "italic"},
            {token: "operator", foreground: "#0aa370"}
        ],
        colors: {
            "editor.background": "#0b0b0e"
        }
    });
};

export interface ServerFileEditorRef
{
    saveContent: () => Promise<void>;
}

interface ServerFileEditorProps
{
    isEditingFile: boolean;
    selectedEntries: FilesystemEntry[];
    selectedFileContents: string;
    browserWidth: number;
    containerRef: React.RefObject<HTMLDivElement>;
    isDragging: boolean;
    needsToSave: boolean;
    onContentChange: (content: string) => void;
    onSave: () => Promise<void>;
    onWidthChange: (width: number) => void;
    onDragStart: () => void;
    onDragEnd: () => void;
}

export const ServerFileEditor = forwardRef<ServerFileEditorRef, ServerFileEditorProps>(
    ({
         isEditingFile,
         selectedEntries,
         selectedFileContents,
         browserWidth,
         containerRef,
         isDragging,
         onContentChange,
         onSave,
         onWidthChange,
         onDragStart,
         onDragEnd
     }, ref) =>
    {
        const editorRef = useRef<any>(null);
        const monacoRef = useRef<any>(null);
        const editorWrapperRef = useRef<HTMLDivElement>(null);

        // Expose saveContent method to parent component
        useImperativeHandle(ref, () => ({
            saveContent: onSave
        }));

        const handleEditorMount = useCallback((editor: any, monaco: any) =>
        {
            editorRef.current = editor;
            monacoRef.current = monaco;

            // Define theme first
            defineObsidianTheme(monaco);

            // Register the Minecraft properties language
            registerMinecraftPropertiesLanguage(monaco);

            // Set the theme after it's defined
            monaco.editor.setTheme("obsidian-editor-theme");

            // Override the toggle line comment keybinding
            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyC,
                () =>
                {
                    editor.trigger("keyboard", "editor.action.commentLine", {});
                }
            );

            // Optional: Disable the original Ctrl+/ keybinding
            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyCode.Slash,
                () =>
                {
                }
            );

            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
                () =>
                {
                    editor.trigger("keyboard", "editor.action.formatDocument", {});
                }
            );

            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS,
                async () =>
                {
                    console.log("Saving content from editor");
                    await onSave();
                }
            );

            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD,
                () =>
                {
                    editor.trigger("keyboard", "editor.action.deleteLines", {});
                }
            );

            editor.addCommand(
                monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyD,
                () =>
                {
                    editor.trigger("keyboard", "editor.action.duplicateSelection", {});
                }
            );
        }, [onSave]);

        return (
            <>
                <motion.div
                    id={"server-file-editor"}
                    ref={editorWrapperRef}
                    className={"max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative flex-1 w-0"}
                    initial={{opacity: 0}}
                    animate={{
                        opacity: isEditingFile && selectedEntries.length === 1 ? 1 : 0
                    }}
                    exit={{opacity: 0}}
                    transition={{duration: isDragging ? 0 : 0.3, ease: "easeInOut"}}
                    data-editing-file={isEditingFile && selectedEntries.length === 1}
                    style={{
                        display: isEditingFile && selectedEntries.length === 1 ? 'block' : 'none'
                    }}
                >
                    {isEditingFile && selectedEntries.length === 1 && isTextFile(selectedEntries[0].path) ? (
                        <Editor
                            className={"w-full h-full"}
                            theme={"obsidian-editor-theme"}
                            value={selectedFileContents}
                            language={getMonacoLanguage(selectedEntries[0]?.path ?? "") ?? "auto"}
                            onMount={handleEditorMount}
                            onChange={async content =>
                            {
                                console.log("Editor content changed:", content);
                                onContentChange(content ?? "");
                            }}
                            options={{
                                fontSize: 14,
                                minimap: {enabled: false},
                                lineNumbers: "on",
                                scrollBeyondLastLine: false,
                                automaticLayout: true,
                                wordWrap: "on",
                                tabSize: 2,
                                contextmenu: false,
                                autoClosingBrackets: "always",
                                autoClosingOvertype: "always",
                                autoClosingQuotes: "always",
                                quickSuggestions: {
                                    other: true,
                                    comments: false,
                                    strings: true
                                },
                                suggestOnTriggerCharacters: true,
                                acceptSuggestionOnEnter: "on",
                                tabCompletion: "on",
                                wordBasedSuggestions: "matchingDocuments",
                                parameterHints: {
                                    enabled: true,
                                    cycle: true
                                },
                                formatOnPaste: true,
                                formatOnType: true,
                                matchBrackets: "always",
                                autoIndent: "full",
                                folding: true,
                                foldingStrategy: "indentation",
                                suggest: {
                                    showKeywords: true,
                                    showSnippets: true,
                                    showFunctions: true,
                                    showConstructors: true,
                                    showFields: true,
                                    showVariables: true,
                                    showClasses: true,
                                    showStructs: true,
                                    showInterfaces: true,
                                    showModules: true,
                                    showProperties: true,
                                    showEvents: true,
                                    showOperators: true,
                                    showUnits: true,
                                    showValues: true,
                                    showConstants: true,
                                    showEnums: true,
                                    showEnumMembers: true,
                                    showColors: true,
                                    showFiles: true,
                                    showReferences: true,
                                    showFolders: true,
                                    showTypeParameters: true,
                                    showUsers: true,
                                    showIssues: true
                                }
                            }}
                        />
                    ) : selectedEntries.length === 1 && !isTextFile(selectedEntries[0].path) ? (
                        <div className="flex items-center justify-center h-full">
                            <span className="text-gray-500 font-minecraft-body">Select a text file to edit</span>
                        </div>
                    ) : null}

                    {/* Resize Handle */}
                    {isEditingFile && selectedEntries.length === 1 && (
                        <div
                            className={cn(
                                "w-[8px] h-full bg-transparent transition-all duration-200 absolute left-0 top-0 cursor-ew-resize select-none hover:bg-primary hover:opacity-50"
                            )}
                            data-dragging={isDragging}
                            onMouseDown={(e) =>
                            {
                                e.preventDefault();
                                onDragStart();

                                const startX = e.clientX;
                                const startWidth = browserWidth;
                                const containerWidth = containerRef.current?.clientWidth;

                                const onMouseMove = (moveEvent: MouseEvent) =>
                                {
                                    moveEvent.preventDefault();
                                    const deltaX = moveEvent.clientX - startX;
                                    const newWidth = startWidth + deltaX;
                                    if (!containerWidth) return;
                                    // Browser can be between 300px and container width - 300px (to leave room for editor)
                                    // Account for the gap-2 (8px) between browser and editor
                                    const gap = 8;
                                    onWidthChange(Math.min(containerWidth - 300 - gap, Math.max(300, newWidth)));
                                };

                                const onMouseUp = (mouseEvent: MouseEvent) =>
                                {
                                    mouseEvent.preventDefault();
                                    onDragEnd();

                                    const deltaX = mouseEvent.clientX - startX;
                                    const gap = 8;
                                    const newWidth = Math.min(containerWidth! - 300 - gap, Math.max(300, startWidth + deltaX));
                                    onWidthChange(newWidth)
                                    localStorage.setItem("browser-width", newWidth.toString());
                                    document.removeEventListener("mousemove", onMouseMove);
                                    document.removeEventListener("mouseup", onMouseUp);
                                };

                                document.addEventListener("mousemove", onMouseMove);
                                document.addEventListener("mouseup", onMouseUp);
                            }}
                        >
                            <span
                                className={cn(
                                    "w-px h-full bg-white opacity-20 transition-all duration-200 absolute left-0 top-0 cursor-ew-resize select-none",
                                    "hover:opacity-50 hover:bg-primary",
                                    "data-[dragging=true]:opacity-50 data-[dragging=true]:bg-primary"
                                )}
                            />
                        </div>
                    )}
                </motion.div>


                {/* Overlay to prevent clicks during dragging */}
                <AnimatePresence>
                    {isDragging && (
                        <motion.div
                            className="fixed inset-0 z-50 cursor-ew-resize select-none pointer-events-auto bg-primary/10"
                            initial={{opacity: 0}}
                            animate={{opacity: 1}}
                            exit={{opacity: 0}}
                            transition={{duration: 0.2}}
                            onClick={(e) => e.stopPropagation()}
                        />
                    )}
                </AnimatePresence>
            </>
        );
    }
);

ServerFileEditor.displayName = "ServerFileEditor";
