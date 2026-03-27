import initWasm, {
  parse as wasmParse,
  parse_to_json as wasmParseToJson,
  render_html as wasmRenderHtml,
  render_html_standalone as wasmRenderStandalone,
  set_panic_hook as wasmSetPanicHook,
} from '../vendor/etch-wasm/etch_wasm.js';

import type { ParseResult } from './types.js';

const NOT_INITIALIZED_ERROR =
  'Etch WASM is not initialized. Call initialize() before using parse() or renderHtml().';

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

export function renderStandaloneFromWasm(input: string): string {
  ensureInitialized();
  return wasmRenderStandalone(input);
}

export function parseToJsonFromWasm(input: string): string {
  ensureInitialized();
  return wasmParseToJson(input);
}

async function doInitialize(): Promise<void> {
  const { readFile } = await import('node:fs/promises');
  const wasmUrl = new URL('../vendor/etch-wasm/etch_wasm_bg.wasm', import.meta.url);
  const bytes = await readFile(wasmUrl);

  await initWasm({ module_or_path: bytes });
  wasmSetPanicHook();
  initialized = true;
}

function ensureInitialized(): void {
  if (!initialized) {
    throw new Error(NOT_INITIALIZED_ERROR);
  }
}
