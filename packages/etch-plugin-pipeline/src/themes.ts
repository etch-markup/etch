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

.chapter {
  margin: 2.5rem 0;
  padding-top: 1.5rem;
  border-top: 1px solid var(--etch-border);
}

.chapter:first-child {
  border-top: none;
  padding-top: 0;
}

.chapter-title {
  margin: 0 0 1rem;
  font-size: 1.75em;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.note > :first-child,
.aside > :first-child,
.details-content > :first-child,
.task-list-item__content > :first-child {
  margin-top: 0;
}

.note > :last-child,
.aside > :last-child,
.details-content > :last-child,
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
  cursor: pointer;
}

.spoiler .spoiler-toggle {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.spoiler .spoiler-content {
  background: var(--etch-spoiler-bg);
  color: transparent;
  border-radius: 0.25em;
  padding: 0.05em 0.35em;
  filter: blur(0.35rem);
  user-select: none;
  transition: color 200ms ease, filter 200ms ease, background 200ms ease;
}

.spoiler:hover .spoiler-content {
  filter: blur(0.25rem);
}

.spoiler .spoiler-toggle:checked + .spoiler-content {
  color: inherit;
  background: transparent;
  filter: none;
  cursor: text;
  user-select: text;
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
  font-weight: 600;
  color: var(--etch-muted);
  margin-right: 0.35em;
}

.footnote-label::before {
  content: "[";
}

.footnote-label::after {
  content: "]";
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

const SYSTEM_FONT_STACK =
  "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif";
const SERIF_FONT_STACK =
  "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif";
const WARNING_BG = "rgba(255, 196, 0, 0.12)";
const WARNING_BORDER = "rgba(255, 196, 0, 0.4)";
const THEME_VALUE_COUNT = 27;

function createThemeVariables(values: readonly string[]): Record<string, string> {
  if (values.length !== THEME_VALUE_COUNT) {
    throw new Error(
      `Expected ${THEME_VALUE_COUNT} theme values, received ${values.length}`
    );
  }

  const [
    bg,
    text,
    headingFont,
    bodyFont,
    accent,
    codeBg,
    noteBg,
    noteBorder,
    noteTipBg,
    noteTipBorder,
    noteWarningBg,
    noteWarningBorder,
    noteCautionBg,
    noteCautionBorder,
    noteDangerBg,
    noteDangerBorder,
    spoilerBg,
    kbdBg,
    kbdBorder,
    muted,
    border,
    surface,
    surfaceStrong,
    codeText,
    warningBg,
    warningBorder,
    warningText
  ] = values;

  return {
    "--etch-bg": bg,
    "--etch-text": text,
    "--etch-heading-font": headingFont,
    "--etch-body-font": bodyFont,
    "--etch-accent": accent,
    "--etch-code-bg": codeBg,
    "--etch-note-bg": noteBg,
    "--etch-note-border": noteBorder,
    "--etch-note-tip-bg": noteTipBg,
    "--etch-note-tip-border": noteTipBorder,
    "--etch-note-warning-bg": noteWarningBg,
    "--etch-note-warning-border": noteWarningBorder,
    "--etch-note-caution-bg": noteCautionBg,
    "--etch-note-caution-border": noteCautionBorder,
    "--etch-note-danger-bg": noteDangerBg,
    "--etch-note-danger-border": noteDangerBorder,
    "--etch-spoiler-bg": spoilerBg,
    "--etch-kbd-bg": kbdBg,
    "--etch-kbd-border": kbdBorder,
    "--etch-muted": muted,
    "--etch-border": border,
    "--etch-surface": surface,
    "--etch-surface-strong": surfaceStrong,
    "--etch-code-text": codeText,
    "--etch-warning-bg": warningBg,
    "--etch-warning-border": warningBorder,
    "--etch-warning-text": warningText
  };
}

function defineTheme(
  name: string,
  values: readonly string[],
  options: {
    css?: string;
    darkValues?: readonly string[];
    darkCss?: string;
  } = {}
): EtchTheme {
  return {
    name,
    variables: createThemeVariables(values),
    ...(options.css ? { css: options.css } : {}),
    ...(options.darkValues
      ? {
          darkMode: {
            variables: createThemeVariables(options.darkValues),
            ...(options.darkCss ? { css: options.darkCss } : {})
          }
        }
      : {})
  };
}

const DEFAULT_LIGHT = [
  "#ffffff", "#1a1a1a", SYSTEM_FONT_STACK, SYSTEM_FONT_STACK, "#2563eb", "#f5f5f5",
  "#f0f7ff", "#2563eb", "#f0fdf4", "#16a34a", "#fffbeb", "#d97706",
  "#fff7ed", "#ea580c", "#fef2f2", "#dc2626", "#f5f5f5", "#f5f5f5", "#d1d5db",
  "#52606d", "rgba(148, 163, 184, 0.35)", "rgba(148, 163, 184, 0.08)", "rgba(148, 163, 184, 0.18)", "#1f2937", WARNING_BG, WARNING_BORDER, "#1a1a1a"
] as const;
const DEFAULT_DARK = [
  "#1a1a1a", "#e0e0e0", SYSTEM_FONT_STACK, SYSTEM_FONT_STACK, "#2563eb", "#2a2a2a",
  "#1a2332", "#3b82f6", "#14231a", "#22c55e", "#231f14", "#f59e0b",
  "#231a14", "#f97316", "#231414", "#ef4444", "#2a2a2a", "#2a2a2a", "#4b5563",
  "#cbd5e1", "rgba(148, 163, 184, 0.25)", "rgba(127, 127, 127, 0.12)", "rgba(127, 127, 127, 0.18)", "#f8fafc", WARNING_BG, WARNING_BORDER, "#e0e0e0"
] as const;
const MINIMAL_LIGHT = [
  "#ffffff", "#333333", SYSTEM_FONT_STACK, SYSTEM_FONT_STACK, "#0066cc", "#f0f0f0",
  "#eef6ff", "#0066cc", "#eefaf1", "#15803d", "#fff8e6", "#ca8a04",
  "#fff1e8", "#ea580c", "#fff1f2", "#dc2626", "#f0f0f0", "#f0f0f0", "#cbd5e1",
  "#5f6b7a", "rgba(148, 163, 184, 0.35)", "rgba(148, 163, 184, 0.08)", "rgba(148, 163, 184, 0.18)", "#111827", WARNING_BG, WARNING_BORDER, "#333333"
] as const;
const MINIMAL_DARK = [
  "#1e1e1e", "#d4d4d4", SYSTEM_FONT_STACK, SYSTEM_FONT_STACK, "#0066cc", "#2d2d2d",
  "#142033", "#60a5fa", "#132318", "#22c55e", "#2a2414", "#fbbf24",
  "#2a1d16", "#fb923c", "#2b161b", "#f87171", "#2d2d2d", "#2d2d2d", "#475569",
  "#cbd5e1", "rgba(148, 163, 184, 0.25)", "rgba(127, 127, 127, 0.12)", "rgba(127, 127, 127, 0.18)", "#f8fafc", WARNING_BG, WARNING_BORDER, "#d4d4d4"
] as const;
const ACADEMIC_LIGHT = [
  "#fcfcfa", "#171717", SERIF_FONT_STACK, SERIF_FONT_STACK, "#1f4d7a", "#f4f4f1",
  "#f3f6fb", "#5a7ea6", "#f2f6f0", "#5d7a4a", "#fbf5e8", "#a9781f",
  "#f9efe7", "#b36b3d", "#f8e9e8", "#a64b4b", "#ece9e1", "#ece9e1", "#c7c0b2",
  "#5b6170", "rgba(115, 130, 155, 0.3)", "rgba(115, 130, 155, 0.08)", "rgba(115, 130, 155, 0.16)", "#171717", WARNING_BG, WARNING_BORDER, "#171717"
] as const;
const ACADEMIC_DARK = [
  "#191a1d", "#e6e7eb", SERIF_FONT_STACK, SERIF_FONT_STACK, "#1f4d7a", "#23252a",
  "#1d2430", "#89a5c6", "#1a221d", "#8ba978", "#292317", "#d3a34a",
  "#2b2018", "#d08a59", "#2b1d1d", "#d78888", "#23252a", "#23252a", "#5d6572",
  "#c6ccd8", "rgba(148, 163, 184, 0.24)", "rgba(127, 127, 127, 0.12)", "rgba(127, 127, 127, 0.18)", "#f5f4ef", WARNING_BG, WARNING_BORDER, "#e6e7eb"
] as const;
const PAPER_LIGHT = [
  "#ffffff", "#111111", SERIF_FONT_STACK, SERIF_FONT_STACK, "#1c3d6e", "#f7f7f4",
  "#f7f8fb", "#4f6f9a", "#f5f8f2", "#5d7a4a", "#fbf6ea", "#9e7629",
  "#faf0e8", "#b56f42", "#f9eceb", "#a64f4f", "#ece9e1", "#f2f2ee", "#cfc8bc",
  "#5a6068", "rgba(17, 17, 17, 0.18)", "rgba(17, 17, 17, 0.04)", "rgba(17, 17, 17, 0.08)", "#111111", WARNING_BG, WARNING_BORDER, "#111111"
] as const;
const FANCY_LIGHT = [
  "#fffff8", "#111111", SERIF_FONT_STACK, SERIF_FONT_STACK, "#8b0000", "#f5f2eb",
  "#f8f1e7", "#8b0000", "#f3f1e7", "#5b6b3a", "#fbf3df", "#b7791f",
  "#f8eadf", "#c05621", "#f8e3e0", "#9b2c2c", "#f1ece2", "#f1ece2", "#c9b79c",
  "#52606d", "rgba(148, 163, 184, 0.35)", "rgba(148, 163, 184, 0.08)", "rgba(148, 163, 184, 0.18)", "#111111", WARNING_BG, WARNING_BORDER, "#111111"
] as const;
const FANCY_DARK = [
  "#1a1a18", "#d4d0c8", SERIF_FONT_STACK, SERIF_FONT_STACK, "#8b0000", "#2a2820",
  "#2a221f", "#c77d7d", "#202319", "#8fbc8f", "#2b2416", "#d9a441",
  "#2c1f18", "#d4884a", "#2c1c1c", "#e07a7a", "#2a2820", "#2a2820", "#6b6253",
  "#c8c0b3", "rgba(148, 163, 184, 0.25)", "rgba(127, 127, 127, 0.12)", "rgba(127, 127, 127, 0.18)", "#f5f2eb", WARNING_BG, WARNING_BORDER, "#d4d0c8"
] as const;

export const BUILTIN_THEMES: Record<string, EtchTheme> = {
  default: defineTheme("default", DEFAULT_LIGHT, {
    css: SHARED_DIRECTIVE_CSS,
    darkValues: DEFAULT_DARK
  }),
  minimal: defineTheme("minimal", MINIMAL_LIGHT, {
    css: SHARED_DIRECTIVE_CSS,
    darkValues: MINIMAL_DARK
  }),
  academic: defineTheme("academic", ACADEMIC_LIGHT, {
    css: SHARED_DIRECTIVE_CSS,
    darkValues: ACADEMIC_DARK
  }),
  paper: defineTheme("paper", PAPER_LIGHT, {
    css: `${SHARED_DIRECTIVE_CSS}
html {
  color-scheme: light;
}`
  }),
  fancy: defineTheme("fancy", FANCY_LIGHT, {
    css: SHARED_DIRECTIVE_CSS,
    darkValues: FANCY_DARK
  })
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
