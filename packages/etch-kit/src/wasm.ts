import initWasm, {
  parse as wasmParse,
  render_html as wasmRenderHtml,
  render_html_document as wasmRenderDocument,
} from 'etch-wasm';

import type { ParseResult } from './types.js';

type ImportMetaWithResolve = ImportMeta & {
  resolve?: (specifier: string) => string;
};

const NOT_INITIALIZED_ERROR =
  'Etch WASM is not initialized. Call initialize() before using parse() or renderHtml().';

export const DEFAULT_STANDALONE_STYLES = `html {
  color-scheme: light dark;
}

body {
  margin: 0;
  padding: 3rem 1.5rem;
  font-family: Georgia, "Times New Roman", serif;
  line-height: 1.7;
  background:
    radial-gradient(circle at top, rgba(160, 174, 192, 0.14), transparent 45%),
    linear-gradient(180deg, #fcfcfd 0%, #f3f4f6 100%);
  color: #1f2933;
}

main {
  max-width: 72ch;
  margin: 0 auto;
}

h1, h2, h3, h4, h5, h6 {
  line-height: 1.2;
  margin: 2rem 0 1rem;
}

p, ul, ol, blockquote, pre, table, dl {
  margin: 1rem 0;
}

a {
  color: #0f5ea8;
}

code, pre {
  font-family: "Cascadia Code", "SFMono-Regular", Consolas, monospace;
}

pre {
  padding: 1rem;
  overflow-x: auto;
  border-radius: 0.75rem;
  background: rgba(15, 23, 42, 0.92);
  color: #e5edf5;
}

code {
  padding: 0.1rem 0.3rem;
  border-radius: 0.35rem;
  background: rgba(148, 163, 184, 0.18);
}

pre code {
  padding: 0;
  background: transparent;
}

blockquote {
  margin-left: 0;
  padding-left: 1rem;
  border-left: 4px solid rgba(15, 94, 168, 0.35);
  color: #52606d;
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
  background: rgba(226, 232, 240, 0.7);
}

img {
  max-width: 100%;
  height: auto;
}

.footnote {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid rgba(148, 163, 184, 0.35);
}

.directive-label {
  font-weight: 700;
  letter-spacing: 0.02em;
}

@media (prefers-color-scheme: dark) {
  body {
    background:
      radial-gradient(circle at top, rgba(96, 165, 250, 0.12), transparent 45%),
      linear-gradient(180deg, #0f172a 0%, #111827 100%);
    color: #e5e7eb;
  }

  a {
    color: #7dd3fc;
  }

  code {
    background: rgba(148, 163, 184, 0.2);
  }

  blockquote {
    color: #cbd5e1;
    border-left-color: rgba(125, 211, 252, 0.45);
  }

  th {
    background: rgba(30, 41, 59, 0.85);
  }

  th, td, .footnote {
    border-color: rgba(148, 163, 184, 0.25);
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
  if (shouldResolveWasmExplicitly()) {
    await initializeWithResolvedModule();
    initialized = true;
    return;
  }

  try {
    await initWasm();
  } catch (error) {
    await initializeWithResolvedModule(error);
  }

  initialized = true;
}

function shouldResolveWasmExplicitly(): boolean {
  return (
    typeof process !== 'undefined' &&
    typeof process.versions === 'object' &&
    typeof process.versions?.node === 'string'
  );
}

async function initializeWithResolvedModule(originalError?: unknown): Promise<void> {
  const wasmUrl = resolveWasmUrl();

  try {
    if (wasmUrl.protocol === 'file:') {
      const { readFile } = await import('node:fs/promises');
      const bytes = await readFile(wasmUrl);
      await initWasm({ module_or_path: bytes });
      return;
    }

    await initWasm({ module_or_path: wasmUrl });
  } catch (error) {
    throw new Error('Failed to initialize etch-wasm.', {
      cause: error ?? originalError,
    });
  }
}

function resolveWasmUrl(): URL {
  const resolver = (import.meta as ImportMetaWithResolve).resolve;

  if (typeof resolver === 'function') {
    return new URL('etch_wasm_bg.wasm', resolver('etch-wasm/etch_wasm.js'));
  }

  return new URL('../../../crates/etch-wasm/pkg/etch_wasm_bg.wasm', import.meta.url);
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
