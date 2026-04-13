import type { EtchDocument, ParseResult } from './types.js';

const DEFAULT_NOT_INITIALIZED_ERROR =
  'Etch WASM is not initialized. Call initialize() before using parse() or renderHtml().';

export type InitializeEtchWasmOptions = {
  wasmUrl?: string | URL;
};

export type EtchWasmBindings = {
  initialize: (options?: InitializeEtchWasmOptions) => void | Promise<void>;
  parse: (input: string) => ParseResult;
  renderHtml: (input: string) => string;
  renderDocument: (input: string) => string;
};

export type CreateEtchKitRuntimeOptions = {
  defaultStandaloneStyles?: string;
  notInitializedError?: string;
};

export type EtchKitRuntime = {
  DEFAULT_STANDALONE_STYLES: string;
  initialize: (options?: InitializeEtchWasmOptions) => Promise<void>;
  parse: (input: string) => EtchDocument;
  parseWithErrors: (input: string) => ParseResult;
  renderHtml: (input: string) => string;
  renderDocument: (input: string) => string;
  renderStandalone: (input: string, styles?: string) => string;
  parseToJson: (input: string) => string;
};

export function createEtchKitRuntime(
  bindings: EtchWasmBindings,
  options: CreateEtchKitRuntimeOptions = {}
): EtchKitRuntime {
  const defaultStandaloneStyles = options.defaultStandaloneStyles ?? '';
  const notInitializedError =
    options.notInitializedError ?? DEFAULT_NOT_INITIALIZED_ERROR;

  let initialized = false;
  let initializePromise: Promise<void> | undefined;

  async function initialize(options?: InitializeEtchWasmOptions): Promise<void> {
    if (initialized) {
      return;
    }

    initializePromise ??= Promise.resolve(bindings.initialize(options)).then(
      () => {
        initialized = true;
      },
      (error: unknown) => {
        initializePromise = undefined;
        throw error;
      }
    );

    await initializePromise;
  }

  function ensureInitialized(): void {
    if (!initialized) {
      throw new Error(notInitializedError);
    }
  }

  function parseWithErrors(input: string): ParseResult {
    ensureInitialized();
    return bindings.parse(input);
  }

  function renderHtml(input: string): string {
    ensureInitialized();
    return bindings.renderHtml(input);
  }

  function renderDocument(input: string): string {
    ensureInitialized();
    return bindings.renderDocument(input);
  }

  function renderStandalone(
    input: string,
    styles: string = defaultStandaloneStyles
  ): string {
    return injectStyles(renderDocument(input), styles);
  }

  function parseToJson(input: string): string {
    return JSON.stringify(serializeForJson(parseWithErrors(input)));
  }

  return {
    DEFAULT_STANDALONE_STYLES: defaultStandaloneStyles,
    initialize,
    parse(input: string): EtchDocument {
      return parseWithErrors(input).document;
    },
    parseWithErrors,
    renderHtml,
    renderDocument,
    renderStandalone,
    parseToJson,
  };
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
      Array.from(value.entries(), ([key, entryValue]) => [
        key,
        serializeForJson(entryValue),
      ])
    );
  }

  if (Array.isArray(value)) {
    return value.map((entry) => serializeForJson(entry));
  }

  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, entryValue]) => [
        key,
        serializeForJson(entryValue),
      ])
    );
  }

  return value;
}
