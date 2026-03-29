import type { EtchTheme } from "@etch-markup/etch-plugin-sdk";

const SHARED_DIRECTIVE_CSS = `
.directive-label {
  font-weight: 700;
  letter-spacing: 0.02em;
}

.note {
  margin: 1rem 0;
  padding: 0.65rem 1rem;
  border-left: 4px solid var(--etch-note-border);
  background: var(--etch-note-bg);
  border-radius: 0 0.5rem 0.5rem 0;
}

.note-label {
  font-weight: 700;
  margin: 0 0 0.5rem;
}

.note--info {
  border-left-color: var(--etch-note-border);
  background: var(--etch-note-bg);
}

.note--tip {
  border-left-color: var(--etch-note-tip-border);
  background: var(--etch-note-tip-bg);
}

.note--warning {
  border-left-color: var(--etch-note-warning-border);
  background: var(--etch-note-warning-bg);
}

.note--caution {
  border-left-color: var(--etch-note-caution-border);
  background: var(--etch-note-caution-bg);
}

.note--danger {
  border-left-color: var(--etch-note-danger-border);
  background: var(--etch-note-danger-bg);
}

.aside {
  margin: 1rem 0;
  padding: 0.65rem 1rem;
  border-left: 3px solid var(--etch-accent);
  font-style: italic;
}

.note > :first-child,
.aside > :first-child,
.details-content > :first-child,
.spoiler-content > :first-child,
.task-list-item__content > :first-child {
  margin-top: 0;
}

.note > :last-child,
.aside > :last-child,
.details-content > :last-child,
.spoiler-content > :last-child,
.task-list-item__content > :last-child {
  margin-bottom: 0;
}

figure {
  margin: 1.5rem 0;
  text-align: center;
}

figcaption {
  margin-top: 0.5rem;
  font-size: 0.9em;
  color: var(--etch-text);
  opacity: 0.7;
}

details {
  margin: 1rem 0;
  border: 1px solid var(--etch-kbd-border);
  border-radius: 0.5rem;
}

details > summary {
  padding: 0.6rem 1rem;
  cursor: pointer;
  font-weight: 600;
}

details[open] > summary {
  border-bottom: 1px solid var(--etch-kbd-border);
}

.details-content {
  padding: 0.75rem 1rem 0.85rem;
}

.spoiler {
  margin: 1rem 0;
}

.spoiler-toggle {
  position: absolute;
  inline-size: 1px;
  block-size: 1px;
  opacity: 0;
  pointer-events: none;
}

.spoiler-card {
  display: block;
  padding: 0.75rem 1rem;
  border: 1px solid var(--etch-kbd-border);
  border-radius: 0.5rem;
}

.spoiler-label {
  margin: 0;
  font-weight: 600;
}

.spoiler-content {
  position: relative;
  margin-top: 0.65rem;
  padding: 0.35rem 0.5rem;
  border-radius: 0.35rem;
  background: var(--etch-spoiler-bg);
  color: transparent;
  filter: blur(0.38rem);
  user-select: none;
  transition: color 140ms ease, filter 140ms ease;
}

.spoiler-content > * {
  visibility: hidden;
}

.spoiler-overlay {
  position: absolute;
  inset: 0;
  z-index: 1;
  cursor: pointer;
  color: transparent;
}

.spoiler-overlay::after {
  content: "Click to reveal";
  position: absolute;
  inset: auto 0.5rem 0.35rem auto;
  color: var(--etch-muted);
  font-size: 0.85em;
  letter-spacing: 0.01em;
}

.spoiler-toggle:focus-visible + .spoiler-card {
  outline: 2px solid var(--etch-accent);
  outline-offset: 2px;
}

.spoiler-toggle:checked + .spoiler-card .spoiler-content {
  color: inherit;
  filter: none;
  user-select: text;
}

.spoiler-toggle:checked + .spoiler-card .spoiler-content > * {
  visibility: visible;
}

.spoiler-toggle:checked + .spoiler-card .spoiler-overlay {
  display: none;
}

.task-list {
  padding-left: 0;
  list-style: none;
}

.task-list-item__body {
  display: flex;
  align-items: flex-start;
  gap: 0.7rem;
}

.task-list-item__checkbox {
  margin: 0.2rem 0 0;
  flex: none;
}

.task-list-item__content {
  flex: 1;
  min-width: 0;
}

.footnote-label {
  margin: 0 0 0.5rem;
  color: var(--etch-muted);
}

.footnote-label sup {
  font-weight: 600;
}

.columns {
  display: grid;
  grid-template-columns: repeat(var(--columns-count, 2), 1fr);
  gap: var(--columns-gap, 1rem);
}

.toc {
  margin: 1rem 0;
}

.toc ol {
  padding-left: 1.5rem;
}

.toc a {
  color: var(--etch-accent);
  text-decoration: none;
}

.toc a:hover {
  text-decoration: underline;
}

.page-break {
  break-after: page;
}

abbr {
  text-decoration: underline dotted;
  cursor: help;
}

kbd {
  display: inline-block;
  padding: 0.15rem 0.4rem;
  font-size: 0.85em;
  font-family: inherit;
  background: var(--etch-kbd-bg);
  border: 1px solid var(--etch-kbd-border);
  border-radius: 0.25rem;
  box-shadow: 0 1px 0 var(--etch-kbd-border);
}

cite {
  font-style: italic;
}

.etch-missing-plugin {
  display: grid;
  gap: 0.35rem;
  margin: 1rem 0;
  padding: 0.85rem 1rem;
  border: 1px solid var(--etch-border);
  border-radius: 0.75rem;
  background: var(--etch-surface);
  color: var(--etch-text);
}

.etch-missing-plugin code {
  padding: 0.1rem 0.35rem;
  border-radius: 0.3rem;
  background: var(--etch-surface-strong);
  color: var(--etch-code-text);
}
`;

export const BUILTIN_THEMES: Record<string, EtchTheme> = {
  default: {
    name: "default",
    variables: {
      "--etch-bg": "#ffffff",
      "--etch-text": "#1a1a1a",
      "--etch-heading-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-body-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-accent": "#2563eb",
      "--etch-code-bg": "#f5f5f5",
      "--etch-note-bg": "#f0f7ff",
      "--etch-note-border": "#2563eb",
      "--etch-note-tip-bg": "#f0fdf4",
      "--etch-note-tip-border": "#16a34a",
      "--etch-note-warning-bg": "#fffbeb",
      "--etch-note-warning-border": "#d97706",
      "--etch-note-caution-bg": "#fff7ed",
      "--etch-note-caution-border": "#ea580c",
      "--etch-note-danger-bg": "#fef2f2",
      "--etch-note-danger-border": "#dc2626",
      "--etch-spoiler-bg": "#f5f5f5",
      "--etch-kbd-bg": "#f5f5f5",
      "--etch-kbd-border": "#d1d5db",
      "--etch-muted": "#52606d",
      "--etch-border": "rgba(148, 163, 184, 0.35)",
      "--etch-surface": "rgba(148, 163, 184, 0.08)",
      "--etch-surface-strong": "rgba(148, 163, 184, 0.18)",
      "--etch-code-text": "#1f2937",
      "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
      "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
      "--etch-warning-text": "#1a1a1a"
    },
    css: SHARED_DIRECTIVE_CSS,
    darkMode: {
      variables: {
        "--etch-bg": "#1a1a1a",
        "--etch-text": "#e0e0e0",
        "--etch-code-bg": "#2a2a2a",
        "--etch-note-bg": "#1a2332",
        "--etch-note-border": "#3b82f6",
        "--etch-note-tip-bg": "#14231a",
        "--etch-note-tip-border": "#22c55e",
        "--etch-note-warning-bg": "#231f14",
        "--etch-note-warning-border": "#f59e0b",
        "--etch-note-caution-bg": "#231a14",
        "--etch-note-caution-border": "#f97316",
        "--etch-note-danger-bg": "#231414",
        "--etch-note-danger-border": "#ef4444",
        "--etch-spoiler-bg": "#2a2a2a",
        "--etch-kbd-bg": "#2a2a2a",
        "--etch-kbd-border": "#4b5563",
        "--etch-muted": "#cbd5e1",
        "--etch-border": "rgba(148, 163, 184, 0.25)",
        "--etch-surface": "rgba(127, 127, 127, 0.12)",
        "--etch-surface-strong": "rgba(127, 127, 127, 0.18)",
        "--etch-code-text": "#f8fafc",
        "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
        "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
        "--etch-warning-text": "#e0e0e0"
      }
    }
  },
  minimal: {
    name: "minimal",
    variables: {
      "--etch-bg": "#ffffff",
      "--etch-text": "#333333",
      "--etch-heading-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-body-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-accent": "#0066cc",
      "--etch-code-bg": "#f0f0f0",
      "--etch-note-bg": "#eef6ff",
      "--etch-note-border": "#0066cc",
      "--etch-note-tip-bg": "#eefaf1",
      "--etch-note-tip-border": "#15803d",
      "--etch-note-warning-bg": "#fff8e6",
      "--etch-note-warning-border": "#ca8a04",
      "--etch-note-caution-bg": "#fff1e8",
      "--etch-note-caution-border": "#ea580c",
      "--etch-note-danger-bg": "#fff1f2",
      "--etch-note-danger-border": "#dc2626",
      "--etch-spoiler-bg": "#f0f0f0",
      "--etch-kbd-bg": "#f0f0f0",
      "--etch-kbd-border": "#cbd5e1",
      "--etch-muted": "#5f6b7a",
      "--etch-border": "rgba(148, 163, 184, 0.35)",
      "--etch-surface": "rgba(148, 163, 184, 0.08)",
      "--etch-surface-strong": "rgba(148, 163, 184, 0.18)",
      "--etch-code-text": "#111827",
      "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
      "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
      "--etch-warning-text": "#333333"
    },
    css: SHARED_DIRECTIVE_CSS,
    darkMode: {
      variables: {
        "--etch-bg": "#1e1e1e",
        "--etch-text": "#d4d4d4",
        "--etch-code-bg": "#2d2d2d",
        "--etch-note-bg": "#142033",
        "--etch-note-border": "#60a5fa",
        "--etch-note-tip-bg": "#132318",
        "--etch-note-tip-border": "#22c55e",
        "--etch-note-warning-bg": "#2a2414",
        "--etch-note-warning-border": "#fbbf24",
        "--etch-note-caution-bg": "#2a1d16",
        "--etch-note-caution-border": "#fb923c",
        "--etch-note-danger-bg": "#2b161b",
        "--etch-note-danger-border": "#f87171",
        "--etch-spoiler-bg": "#2d2d2d",
        "--etch-kbd-bg": "#2d2d2d",
        "--etch-kbd-border": "#475569",
        "--etch-muted": "#cbd5e1",
        "--etch-border": "rgba(148, 163, 184, 0.25)",
        "--etch-surface": "rgba(127, 127, 127, 0.12)",
        "--etch-surface-strong": "rgba(127, 127, 127, 0.18)",
        "--etch-code-text": "#f8fafc",
        "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
        "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
        "--etch-warning-text": "#d4d4d4"
      }
    }
  },
  academic: {
    name: "academic",
    variables: {
      "--etch-bg": "#fcfcfa",
      "--etch-text": "#171717",
      "--etch-heading-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-body-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-accent": "#1f4d7a",
      "--etch-code-bg": "#f4f4f1",
      "--etch-note-bg": "#f3f6fb",
      "--etch-note-border": "#5a7ea6",
      "--etch-note-tip-bg": "#f2f6f0",
      "--etch-note-tip-border": "#5d7a4a",
      "--etch-note-warning-bg": "#fbf5e8",
      "--etch-note-warning-border": "#a9781f",
      "--etch-note-caution-bg": "#f9efe7",
      "--etch-note-caution-border": "#b36b3d",
      "--etch-note-danger-bg": "#f8e9e8",
      "--etch-note-danger-border": "#a64b4b",
      "--etch-spoiler-bg": "#ece9e1",
      "--etch-kbd-bg": "#ece9e1",
      "--etch-kbd-border": "#c7c0b2",
      "--etch-muted": "#5b6170",
      "--etch-border": "rgba(115, 130, 155, 0.3)",
      "--etch-surface": "rgba(115, 130, 155, 0.08)",
      "--etch-surface-strong": "rgba(115, 130, 155, 0.16)",
      "--etch-code-text": "#171717",
      "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
      "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
      "--etch-warning-text": "#171717"
    },
    css: SHARED_DIRECTIVE_CSS,
    darkMode: {
      variables: {
        "--etch-bg": "#191a1d",
        "--etch-text": "#e6e7eb",
        "--etch-code-bg": "#23252a",
        "--etch-note-bg": "#1d2430",
        "--etch-note-border": "#89a5c6",
        "--etch-note-tip-bg": "#1a221d",
        "--etch-note-tip-border": "#8ba978",
        "--etch-note-warning-bg": "#292317",
        "--etch-note-warning-border": "#d3a34a",
        "--etch-note-caution-bg": "#2b2018",
        "--etch-note-caution-border": "#d08a59",
        "--etch-note-danger-bg": "#2b1d1d",
        "--etch-note-danger-border": "#d78888",
        "--etch-spoiler-bg": "#23252a",
        "--etch-kbd-bg": "#23252a",
        "--etch-kbd-border": "#5d6572",
        "--etch-muted": "#c6ccd8",
        "--etch-border": "rgba(148, 163, 184, 0.24)",
        "--etch-surface": "rgba(127, 127, 127, 0.12)",
        "--etch-surface-strong": "rgba(127, 127, 127, 0.18)",
        "--etch-code-text": "#f5f4ef",
        "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
        "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
        "--etch-warning-text": "#e6e7eb"
      }
    }
  },
  paper: {
    name: "paper",
    variables: {
      "--etch-bg": "#ffffff",
      "--etch-text": "#111111",
      "--etch-heading-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-body-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-accent": "#1c3d6e",
      "--etch-code-bg": "#f7f7f4",
      "--etch-note-bg": "#f7f8fb",
      "--etch-note-border": "#4f6f9a",
      "--etch-note-tip-bg": "#f5f8f2",
      "--etch-note-tip-border": "#5d7a4a",
      "--etch-note-warning-bg": "#fbf6ea",
      "--etch-note-warning-border": "#9e7629",
      "--etch-note-caution-bg": "#faf0e8",
      "--etch-note-caution-border": "#b56f42",
      "--etch-note-danger-bg": "#f9eceb",
      "--etch-note-danger-border": "#a64f4f",
      "--etch-spoiler-bg": "#ece9e1",
      "--etch-kbd-bg": "#f2f2ee",
      "--etch-kbd-border": "#cfc8bc",
      "--etch-muted": "#5a6068",
      "--etch-border": "rgba(17, 17, 17, 0.18)",
      "--etch-surface": "rgba(17, 17, 17, 0.04)",
      "--etch-surface-strong": "rgba(17, 17, 17, 0.08)",
      "--etch-code-text": "#111111",
      "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
      "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
      "--etch-warning-text": "#111111"
    },
    css: `${SHARED_DIRECTIVE_CSS}
html {
  color-scheme: light;
}`,
  },
  fancy: {
    name: "fancy",
    variables: {
      "--etch-bg": "#fffff8",
      "--etch-text": "#111111",
      "--etch-heading-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-body-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-accent": "#8b0000",
      "--etch-code-bg": "#f5f2eb",
      "--etch-note-bg": "#f8f1e7",
      "--etch-note-border": "#8b0000",
      "--etch-note-tip-bg": "#f3f1e7",
      "--etch-note-tip-border": "#5b6b3a",
      "--etch-note-warning-bg": "#fbf3df",
      "--etch-note-warning-border": "#b7791f",
      "--etch-note-caution-bg": "#f8eadf",
      "--etch-note-caution-border": "#c05621",
      "--etch-note-danger-bg": "#f8e3e0",
      "--etch-note-danger-border": "#9b2c2c",
      "--etch-spoiler-bg": "#f1ece2",
      "--etch-kbd-bg": "#f1ece2",
      "--etch-kbd-border": "#c9b79c",
      "--etch-muted": "#52606d",
      "--etch-border": "rgba(148, 163, 184, 0.35)",
      "--etch-surface": "rgba(148, 163, 184, 0.08)",
      "--etch-surface-strong": "rgba(148, 163, 184, 0.18)",
      "--etch-code-text": "#111111",
      "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
      "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
      "--etch-warning-text": "#111111"
    },
    css: SHARED_DIRECTIVE_CSS,
    darkMode: {
      variables: {
        "--etch-bg": "#1a1a18",
        "--etch-text": "#d4d0c8",
        "--etch-code-bg": "#2a2820",
        "--etch-note-bg": "#2a221f",
        "--etch-note-border": "#c77d7d",
        "--etch-note-tip-bg": "#202319",
        "--etch-note-tip-border": "#8fbc8f",
        "--etch-note-warning-bg": "#2b2416",
        "--etch-note-warning-border": "#d9a441",
        "--etch-note-caution-bg": "#2c1f18",
        "--etch-note-caution-border": "#d4884a",
        "--etch-note-danger-bg": "#2c1c1c",
        "--etch-note-danger-border": "#e07a7a",
        "--etch-spoiler-bg": "#2a2820",
        "--etch-kbd-bg": "#2a2820",
        "--etch-kbd-border": "#6b6253",
        "--etch-muted": "#c8c0b3",
        "--etch-border": "rgba(148, 163, 184, 0.25)",
        "--etch-surface": "rgba(127, 127, 127, 0.12)",
        "--etch-surface-strong": "rgba(127, 127, 127, 0.18)",
        "--etch-code-text": "#f5f2eb",
        "--etch-warning-bg": "rgba(255, 196, 0, 0.12)",
        "--etch-warning-border": "rgba(255, 196, 0, 0.4)",
        "--etch-warning-text": "#d4d0c8"
      }
    }
  }
};

export function assembleThemeCSS(theme: EtchTheme): string {
  let css = ":root {\n";
  for (const [key, value] of Object.entries(theme.variables)) {
    css += `  ${key}: ${value};\n`;
  }
  css += "}\n";

  if (theme.css) {
    css += `${theme.css}\n`;
  }

  if (theme.darkMode) {
    css += "@media (prefers-color-scheme: dark) {\n  :root {\n";
    for (const [key, value] of Object.entries(theme.darkMode.variables)) {
      css += `    ${key}: ${value};\n`;
    }
    css += "  }\n";
    if (theme.darkMode.css) {
      css += `${theme.darkMode.css}\n`;
    }
    css += "}\n";
  }

  return css;
}
