import { readFile } from 'node:fs/promises';

import type { ParseResult } from '../../../../packages/etch-kit/dist/index.js';
import { createEtchKitRuntime } from '../../../../packages/etch-kit/dist/runtime.js';
import initWasm, {
  parse as wasmParse,
  render_html as wasmRenderHtml,
  render_html_document as wasmRenderDocument,
} from '../../src/vendor/etch-wasm/etch_wasm.js';

import { VSCODE_PREVIEW_STYLES } from './styles.js';

const runtime = createEtchKitRuntime(
  {
    async initialize() {
      const wasmUrl = new URL('../etch-wasm/etch_wasm_bg.wasm', import.meta.url);
      const bytes = await readFile(wasmUrl);

      await initWasm({ module_or_path: bytes });
    },
    parse(input: string): ParseResult {
      return wasmParse(input) as ParseResult;
    },
    renderHtml: wasmRenderHtml,
    renderDocument: wasmRenderDocument,
  },
  {
    defaultStandaloneStyles: VSCODE_PREVIEW_STYLES,
  }
);

export const DEFAULT_STANDALONE_STYLES = runtime.DEFAULT_STANDALONE_STYLES;
export const initialize = runtime.initialize;
export const parse = runtime.parse;
export const parseWithErrors = runtime.parseWithErrors;
export const renderHtml = runtime.renderHtml;
export const renderDocument = runtime.renderDocument;
export const renderStandalone = runtime.renderStandalone;
export const parseToJson = runtime.parseToJson;
