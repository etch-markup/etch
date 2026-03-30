declare module 'etch-wasm/etch_wasm_loader.js' {
  export default function init(options?: {
    module_or_path?:
      | BufferSource
      | WebAssembly.Module
      | Promise<Response>
      | Request
      | URL
      | string;
  }): Promise<unknown>;

  export function parse(input: string): any;
  export function render_html(input: string): string;
  export function render_html_document(input: string): string;
}
