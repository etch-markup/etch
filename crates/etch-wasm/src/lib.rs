use etch_core::{
    ParseResult, parse as parse_document, render_html as render_html_fragment,
    render_html_document as render_full_html_document,
};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}

fn set_panic_hook() {
    #[cfg(feature = "console-error-panic-hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn parse(input: &str) -> JsValue {
    let result = parse_result(input);
    parse_result_to_js_value(&result)
}

#[wasm_bindgen]
pub fn render_html(input: &str) -> String {
    let result = parse_result(input);
    render_html_fragment(&result.document)
}

#[wasm_bindgen]
pub fn render_html_document(input: &str) -> String {
    let result = parse_result(input);
    render_full_html_document(&result.document)
}

fn parse_result(input: &str) -> ParseResult {
    parse_document(input)
}

fn parse_result_to_js_value(result: &ParseResult) -> JsValue {
    serde_wasm_bindgen::to_value(result).expect("Etch parse result should serialize to JsValue")
}

#[cfg(test)]
mod tests {
    use super::{parse_result, render_html, render_html_document};

    #[test]
    fn render_html_uses_partial_parse_result() {
        let result = parse_result("::aside\nMissing close");
        assert!(!result.errors.is_empty());

        let html = render_html("::aside\nMissing close");
        assert!(html.contains("<aside class=\"aside\">"));
        assert!(html.contains("<p>Missing close</p>"));
    }

    #[test]
    fn render_html_document_returns_full_document_shell() {
        let html = render_html_document("# Hello");

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(!html.contains("<style>"));
        assert!(html.contains("<h1 id=\"hello\">Hello</h1>"));
    }
}
