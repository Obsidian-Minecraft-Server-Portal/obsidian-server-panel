import type { Monaco } from "@monaco-editor/react";

export const OBSIDIAN_THEME = "obsidian";

export function defineObsidianTheme(monaco: Monaco): void {
  monaco.editor.defineTheme(OBSIDIAN_THEME, {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "7A6A80", fontStyle: "italic" },
      { token: "keyword", foreground: "D9A8FF" },
      { token: "string", foreground: "FFB545" },
      { token: "number", foreground: "FF6B3A" },
    ],
    colors: {
      "editor.background": "#151517",
      "editor.foreground": "#EDEDEF",
      "editor.lineHighlightBackground": "#1B1B1E",
      "editor.selectionBackground": "#8A2BE266",
      "editor.inactiveSelectionBackground": "#8A2BE233",
      "editorCursor.foreground": "#B24BFF",
      "editorLineNumber.foreground": "#FFFFFF45",
      "editorLineNumber.activeForeground": "#D9A8FF",
      "editorGutter.background": "#151517",
      "editorWidget.background": "#232327",
      "editorWidget.border": "#FFFFFF1F",
    },
  });
}
