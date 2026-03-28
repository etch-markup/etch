import initWasm, {
  parse as wasmParse,
  render_html as wasmRenderHtml,
  render_html_document as wasmRenderDocument,
} from '../vendor/etch-wasm/etch_wasm.js';

import type { ParseResult } from './types.js';

const NOT_INITIALIZED_ERROR =
  'Etch WASM is not initialized. Call initialize() before using parse() or renderHtml().';

export const DEFAULT_STANDALONE_STYLES = `html {
  color-scheme: light dark;
}

body {
  margin: 0;
  padding: 3rem 1.5rem;
  font-family: var(--etch-body-font, Georgia, "Times New Roman", serif);
  line-height: 1.7;
  background: var(--etch-bg, var(--vscode-editor-background));
  color: var(--etch-text, var(--vscode-editor-foreground));
}

main {
  max-width: 72ch;
  margin: 0 auto;
}

h1, h2, h3, h4, h5, h6 {
  font-family: var(--etch-heading-font, Georgia, "Times New Roman", serif);
  line-height: 1.2;
  margin: 2rem 0 1rem;
}

p, ul, ol, blockquote, pre, table, dl {
  margin: 1rem 0;
}

a {
  color: var(--etch-accent, var(--vscode-textLink-foreground));
}

code, pre {
  font-family: "Cascadia Code", "SFMono-Regular", Consolas, monospace;
}

pre {
  padding: 1rem;
  overflow-x: auto;
  border-radius: 0.75rem;
  background: var(--etch-code-bg, var(--vscode-textCodeBlock-background));
  color: var(--etch-text, var(--vscode-editor-foreground));
}

code {
  padding: 0.1rem 0.3rem;
  border-radius: 0.35rem;
  background: color-mix(in srgb, var(--etch-code-bg, var(--vscode-textCodeBlock-background)) 85%, transparent);
}

pre code {
  padding: 0;
  background: transparent;
}

blockquote {
  margin-left: 0;
  padding-left: 1rem;
  border-left: 4px solid color-mix(in srgb, var(--etch-accent, var(--vscode-textLink-foreground)) 35%, transparent);
  color: var(--etch-text, var(--vscode-editor-foreground));
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  padding: 0.65rem 0.8rem;
  border: 1px solid rgba(148, 163, 184, 0.35);
}

th {
  background: color-mix(in srgb, var(--etch-code-bg, var(--vscode-textCodeBlock-background)) 80%, transparent);
}

img {
  max-width: 100%;
  height: auto;
}

.footnote {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid color-mix(in srgb, var(--etch-text, var(--vscode-editor-foreground)) 20%, transparent);
}

.directive-label {
  font-weight: 700;
  letter-spacing: 0.02em;
}

math {
  font-family: "STIX Two Math", "Cambria Math", serif;
}

math[display="block"] {
  display: block;
  text-align: center;
  margin: 1em 0;
}

.etch-missing-plugin {
  display: grid;
  gap: 0.35rem;
  margin: 1rem 0;
  padding: 0.85rem 1rem;
  border: 1px solid color-mix(in srgb, var(--etch-accent, var(--vscode-textLink-foreground)) 45%, transparent);
  border-radius: 0.75rem;
  background: color-mix(in srgb, var(--etch-code-bg, var(--vscode-textCodeBlock-background)) 85%, transparent);
}

.etch-missing-plugin code {
  background: color-mix(in srgb, var(--etch-code-bg, var(--vscode-textCodeBlock-background)) 92%, transparent);
  padding: 0.1rem 0.35rem;
  border-radius: 0.3rem;
}

@media (prefers-color-scheme: dark) {
  code {
    background: color-mix(in srgb, var(--etch-code-bg, var(--vscode-textCodeBlock-background)) 82%, transparent);
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
  const { readFile } = await import('node:fs/promises');
  const wasmUrl = new URL('../vendor/etch-wasm/etch_wasm_bg.wasm', import.meta.url);
  const bytes = await readFile(wasmUrl);

  await initWasm({ module_or_path: bytes });
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
