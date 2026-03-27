import initWasm, {
  parse as wasmParse,
  parse_to_json as wasmParseToJson,
  render_html as wasmRenderHtml,
  render_html_standalone as wasmRenderStandalone,
  set_panic_hook as wasmSetPanicHook,
} from 'etch-wasm';

import type { ParseResult } from './types.js';

type ImportMetaWithResolve = ImportMeta & {
  resolve?: (specifier: string) => string;
};

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
  if (shouldResolveWasmExplicitly()) {
    await initializeWithResolvedModule();
    wasmSetPanicHook();
    initialized = true;
    return;
  }

  try {
    await initWasm();
  } catch (error) {
    await initializeWithResolvedModule(error);
  }

  wasmSetPanicHook();
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
