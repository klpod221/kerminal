// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import type { Extension } from "@codemirror/state";

/**
 * Kerminal Native Dark Theme for CodeMirror 6
 * Matches Kerminal's Obsidian / Charcoal design system (#0d0d0d / #171717)
 */
export const kerminalBaseTheme = EditorView.theme(
  {
    "&": {
      color: "#e5e5e5",
      backgroundColor: "#121212",
      fontFamily: '"FiraCode Nerd Font", "JetBrains Mono", Menlo, Monaco, Consolas, monospace',
      fontSize: "13px",
    },
    ".cm-content": {
      caretColor: "#38bdf8",
      padding: "8px 0",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "#38bdf8",
      borderLeftWidth: "2px",
    },
    "&.cm-focused .cm-selectionBackground, ::selection": {
      backgroundColor: "rgba(56, 189, 248, 0.2) !important",
    },
    ".cm-gutters": {
      backgroundColor: "#121212",
      color: "#525252",
      borderRight: "1px solid #1f1f1f",
      paddingRight: "8px",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "#1f1f1f",
      color: "#e5e5e5",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(255, 255, 255, 0.03)",
    },
    ".cm-foldPlaceholder": {
      backgroundColor: "#1f1f1f",
      border: "1px solid #333333",
      color: "#38bdf8",
      padding: "0 6px",
      borderRadius: "4px",
    },
    ".cm-tooltip": {
      backgroundColor: "#171717",
      border: "1px solid #333333",
      color: "#e5e5e5",
      borderRadius: "6px",
      boxShadow: "0 4px 12px rgba(0, 0, 0, 0.5)",
    },
    ".cm-tooltip-autocomplete": {
      "& > ul > li[aria-selected]": {
        backgroundColor: "#262626",
        color: "#ffffff",
      },
    },
    ".cm-panels": {
      backgroundColor: "#171717",
      color: "#e5e5e5",
      borderTop: "1px solid #1f1f1f",
    },
  },
  { dark: true },
);

export const kerminalHighlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: "#c084fc", fontWeight: "600" }, // Purple / Mauve
  { tag: [t.name, t.deleted, t.character, t.macroName], color: "#e5e5e5" }, // Text
  { tag: [t.propertyName], color: "#38bdf8" }, // Sky cyan
  { tag: [t.variableName], color: "#e5e5e5" }, // Text
  { tag: [t.function(t.variableName)], color: "#60a5fa" }, // Blue
  { tag: [t.labelName], color: "#f472b6" }, // Pink
  { tag: [t.color, t.constant(t.name), t.standard(t.name)], color: "#fb923c" }, // Orange
  { tag: [t.definition(t.name), t.separator], color: "#e5e5e5" },
  { tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace], color: "#fb923c" }, // Orange
  { tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)], color: "#74c7ec" }, // Cyan
  { tag: [t.meta, t.comment], color: "#6b7280", fontStyle: "italic" }, // Muted gray
  { tag: t.strong, fontWeight: "bold" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  { tag: t.link, color: "#38bdf8", textDecoration: "underline" },
  { tag: t.heading, fontWeight: "bold", color: "#f87171" }, // Red
  { tag: [t.atom, t.bool, t.special(t.variableName)], color: "#fb923c" }, // Orange
  { tag: [t.processingInstruction, t.string, t.inserted], color: "#4ade80" }, // Green
  { tag: t.invalid, color: "#f87171" }, // Red
]);

export const kerminalEditorTheme: Extension = [
  kerminalBaseTheme,
  syntaxHighlighting(kerminalHighlightStyle),
];
