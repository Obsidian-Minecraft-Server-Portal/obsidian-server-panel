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
      "editor.background": "#110B12",
      "editor.foreground": "#EDE6F0",
      "editor.lineHighlightBackground": "#190F18",
      "editor.selectionBackground": "#8A2BE266",
      "editor.inactiveSelectionBackground": "#8A2BE233",
      "editorCursor.foreground": "#B24BFF",
      "editorLineNumber.foreground": "#FFFFFF45",
      "editorLineNumber.activeForeground": "#D9A8FF",
      "editorGutter.background": "#110B12",
      "editorWidget.background": "#231623",
      "editorWidget.border": "#FFFFFF1F",
    },
  });
}
