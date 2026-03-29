import {
  init as initWasm,
  parse as wasmParse,
  render_html as wasmRenderHtml,
  render_html_document as wasmRenderDocument,
} from 'etch-wasm';

import type { ParseResult } from './types.js';

const NOT_INITIALIZED_ERROR =
  'Etch WASM is not initialized. Call initialize() before using parse() or renderHtml().';

export const DEFAULT_STANDALONE_STYLES = `html {
  color-scheme: light dark;
}

body {
  margin: 0;
  padding: clamp(1.25rem, 4vw, 3rem) clamp(1rem, 4vw, 2.25rem);
  font-family: var(
    --etch-body-font,
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    sans-serif
  );
  line-height: 1.7;
  background: var(--etch-bg, #fcfcfd);
  color: var(--etch-text, #1f2933);
}

main {
  width: min(100%, 58rem);
  margin: 0 auto;
}

h1, h2, h3, h4, h5, h6 {
  font-family: var(
    --etch-heading-font,
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    sans-serif
  );
  line-height: 1.2;
  margin: 2rem 0 1rem;
}

p, ul, ol, blockquote, pre, table, dl {
  margin: 1rem 0;
}

a {
  color: var(--etch-accent, #0f5ea8);
}

code, pre {
  font-family: "Cascadia Code", "SFMono-Regular", Consolas, monospace;
}

pre {
  padding: 1rem;
  overflow-x: auto;
  border-radius: 0.75rem;
  background: var(--etch-code-bg, rgba(15, 23, 42, 0.92));
  color: var(--etch-code-text, #e5edf5);
}

code {
  padding: 0.1rem 0.3rem;
  border-radius: 0.35rem;
  background: color-mix(in srgb, var(--etch-code-bg, rgba(15, 23, 42, 0.92)) 85%, transparent);
}

pre code {
  padding: 0;
  background: transparent;
}

blockquote {
  margin-left: 0;
  padding-left: 1rem;
  border-left: 4px solid color-mix(in srgb, var(--etch-accent, #0f5ea8) 35%, transparent);
  color: var(--etch-muted, #52606d);
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  padding: 0.65rem 0.8rem;
  border: 1px solid var(--etch-border, rgba(148, 163, 184, 0.35));
}

th {
  background: color-mix(in srgb, var(--etch-surface-strong, rgba(226, 232, 240, 0.7)) 80%, transparent);
}

img {
  max-width: 100%;
  height: auto;
}

.footnote {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--etch-border, rgba(148, 163, 184, 0.35));
}

.footnote-label {
  margin: 0 0 0.5rem;
  color: var(--etch-muted, #52606d);
}

.footnote-label sup {
  font-weight: 600;
}

.directive-label {
  font-weight: 700;
  letter-spacing: 0.02em;
}

.note {
  margin: 1rem 0;
  padding: 0.65rem 1rem;
  border-left: 4px solid #2563eb;
  background: #f0f7ff;
  border-radius: 0 0.5rem 0.5rem 0;
}

.note-label {
  font-weight: 700;
  margin: 0 0 0.5rem;
}

.note--tip {
  border-left-color: #16a34a;
  background: #f0fdf4;
}

.note--warning {
  border-left-color: #d97706;
  background: #fffbeb;
}

.note--caution {
  border-left-color: #ea580c;
  background: #fff7ed;
}

.note--danger {
  border-left-color: #dc2626;
  background: #fef2f2;
}

.aside {
  margin: 1rem 0;
  padding: 0.65rem 1rem;
  border-left: 3px solid #0f5ea8;
  font-style: italic;
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
  opacity: 0.7;
}

details {
  margin: 1rem 0;
  border: 1px solid rgba(148, 163, 184, 0.35);
  border-radius: 0.5rem;
}

details > summary {
  padding: 0.6rem 1rem;
  cursor: pointer;
  font-weight: 600;
}

details[open] > summary {
  border-bottom: 1px solid rgba(148, 163, 184, 0.35);
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
  background: var(--etch-spoiler-bg, #f5f5f5);
  color: transparent;
  border-radius: 0.25em;
  padding: 0.05em 0.35em;
  user-select: none;
  transition: color 180ms ease, background 180ms ease;
}

.spoiler:hover .spoiler-content {
  background: color-mix(in srgb, var(--etch-spoiler-bg, #f5f5f5) 70%, transparent);
}

.spoiler .spoiler-toggle:checked + .spoiler-content {
  color: inherit;
  background: transparent;
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
  color: #0f5ea8;
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
  background: #f5f5f5;
  border: 1px solid #d1d5db;
  border-radius: 0.25rem;
  box-shadow: 0 1px 0 #d1d5db;
}

cite {
  font-style: italic;
}

.etch-missing-plugin {
  display: grid;
  gap: 0.35rem;
  margin: 1rem 0;
  padding: 0.85rem 1rem;
  border: 1px solid var(--etch-warning-border, rgba(255, 196, 0, 0.4));
  border-radius: 0.75rem;
  background: var(--etch-warning-bg, rgba(255, 196, 0, 0.12));
  color: var(--etch-warning-text, var(--etch-text, #1f2933));
}

.etch-missing-plugin code {
  background: color-mix(in srgb, var(--etch-code-bg, rgba(15, 23, 42, 0.92)) 92%, transparent);
  color: var(--etch-code-text, var(--etch-text, #1f2933));
  padding: 0.1rem 0.35rem;
  border-radius: 0.3rem;
}

math {
  font-family: "STIX Two Math", "Cambria Math", serif;
}

math[display="block"] {
  display: block;
  text-align: center;
  margin: 1em 0;
}

@media (prefers-color-scheme: dark) {
  code {
    background: color-mix(in srgb, var(--etch-code-bg, rgba(15, 23, 42, 0.92)) 82%, transparent);
  }

  body {
    background: var(--etch-bg, #111827);
    color: var(--etch-text, #e5e7eb);
  }

  a {
    color: var(--etch-accent, #7dd3fc);
  }

  blockquote {
    color: var(--etch-muted, #cbd5e1);
    border-left-color: color-mix(in srgb, var(--etch-accent, #7dd3fc) 45%, transparent);
  }

  .note {
    background: #1a2332;
    border-left-color: #3b82f6;
  }

  .note--tip {
    background: #14231a;
    border-left-color: #22c55e;
  }

  .note--warning {
    background: #231f14;
    border-left-color: #f59e0b;
  }

  .note--caution {
    background: #231a14;
    border-left-color: #f97316;
  }

  .note--danger {
    background: #231414;
    border-left-color: #ef4444;
  }

  .aside {
    border-left-color: #7dd3fc;
  }

  kbd {
    background: #2a2a2a;
    border-color: #4b5563;
    box-shadow: 0 1px 0 #4b5563;
  }

  th {
    background: color-mix(in srgb, var(--etch-surface-strong, rgba(30, 41, 59, 0.85)) 80%, transparent);
  }

  th, td, .footnote {
    border-color: var(--etch-border, rgba(148, 163, 184, 0.25));
  }
}`;

let initialized = false;
let initializePromise: Promise<void> | undefined;

export async function initializeWasm(): Promise<void> {
  if (initialized) {
    return;
  }

  if (!initializePromise) {
    initializePromise = doInitialize().catch((error: unknown) => {
      initializePromise = undefined;
      throw error;
    });
  }

  await initializePromise;
}

export function parseResultFromWasm(input: string): ParseResult {
  ensureInitialized();
  return wasmParse(input) as ParseResult;
}

export function renderHtmlFromWasm(input: string): string {
  ensureInitialized();
  return wasmRenderHtml(input);
}

export function renderDocumentFromWasm(input: string): string {
  ensureInitialized();
  return wasmRenderDocument(input);
}

export function renderStandaloneFromWasm(
  input: string,
  styles: string = DEFAULT_STANDALONE_STYLES
): string {
  return injectStyles(renderDocumentFromWasm(input), styles);
}

export function parseToJsonFromWasm(input: string): string {
  return JSON.stringify(serializeForJson(parseResultFromWasm(input)));
}

async function doInitialize(): Promise<void> {
  initWasm();
  initialized = true;
}

function ensureInitialized(): void {
  if (!initialized) {
    throw new Error(NOT_INITIALIZED_ERROR);
  }
}

function injectStyles(documentHtml: string, styles: string): string {
  const styledDocument = `<style>\n${styles}\n</style>\n`;

  if (documentHtml.includes('</head>')) {
    return wrapBodyInMain(documentHtml.replace('</head>', `${styledDocument}</head>`));
  }

  return wrapBodyInMain(`${styledDocument}${documentHtml}`);
}

function wrapBodyInMain(documentHtml: string): string {
  const bodyMarker = '<body>\n';
  const bodyStart = documentHtml.indexOf(bodyMarker);

  if (bodyStart === -1) {
    return documentHtml;
  }

  const insertAt = bodyStart + bodyMarker.length;
  const withMainOpen =
    documentHtml.slice(0, insertAt) + '<main>\n' + documentHtml.slice(insertAt);
  const bodyEnd = withMainOpen.lastIndexOf('\n</body>');

  if (bodyEnd !== -1) {
    return withMainOpen.slice(0, bodyEnd) + '\n</main>' + withMainOpen.slice(bodyEnd);
  }

  const compactBodyEnd = withMainOpen.lastIndexOf('</body>');
  if (compactBodyEnd !== -1) {
    return (
      withMainOpen.slice(0, compactBodyEnd) +
      '</main>\n' +
      withMainOpen.slice(compactBodyEnd)
    );
  }

  return withMainOpen;
}

function serializeForJson(value: unknown): unknown {
  if (value instanceof Map) {
    return Object.fromEntries(
      Array.from(value.entries(), ([key, entryValue]) => [key, serializeForJson(entryValue)])
    );
  }

  if (Array.isArray(value)) {
    return value.map((entry) => serializeForJson(entry));
  }

  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, entryValue]) => [key, serializeForJson(entryValue)])
    );
  }

  return value;
}
