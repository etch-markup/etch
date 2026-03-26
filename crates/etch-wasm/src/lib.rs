use etch_core::{
    ParseResult, parse as parse_document, render_html as render_html_fragment, render_html_document,
};
use serde_json::{Value, json};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

const DEFAULT_STANDALONE_STYLES: &str = r#"html {
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
}"#;

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn parse(input: &str) -> JsValue {
    set_panic_hook();
    let result = parse_result(input);
    parse_result_to_js_value(&result)
}

#[wasm_bindgen]
pub fn render_html(input: &str) -> String {
    set_panic_hook();
    let result = parse_result(input);
    render_html_fragment(&result.document)
}

#[wasm_bindgen]
pub fn render_html_standalone(input: &str) -> String {
    set_panic_hook();
    let result = parse_result(input);
    inject_default_styles(render_html_document(&result.document))
}

#[wasm_bindgen]
pub fn parse_to_json(input: &str) -> String {
    set_panic_hook();
    let result = parse_result(input);
    serialize_parse_result(&result, None)
}

fn parse_result(input: &str) -> ParseResult {
    parse_document(input)
}

fn parse_result_to_js_value(result: &ParseResult) -> JsValue {
    match serde_wasm_bindgen::to_value(result) {
        Ok(value) => value,
        Err(error) => {
            let json = serialize_parse_result(
                result,
                Some(format!(
                    "failed to convert parse result to JsValue: {error}"
                )),
            );

            js_sys::JSON::parse(&json).unwrap_or_else(|_| JsValue::from_str(&json))
        }
    }
}

fn serialize_parse_result(result: &ParseResult, extra_error: Option<String>) -> String {
    if extra_error.is_none() {
        if let Ok(json) = serde_json::to_string(result) {
            return json;
        }
    }

    build_parse_result_value(result, extra_error).to_string()
}

fn build_parse_result_value(result: &ParseResult, extra_error: Option<String>) -> Value {
    let document = serde_json::to_value(&result.document)
        .unwrap_or_else(|_| json!({ "frontmatter": null, "body": [] }));

    let mut errors = match serde_json::to_value(&result.errors) {
        Ok(Value::Array(errors)) => errors,
        Ok(other) => vec![other],
        Err(error) => vec![json!({
            "kind": "Error",
            "message": format!("failed to serialize parser issues: {error}"),
            "line": 0,
            "column": null
        })],
    };

    if let Some(message) = extra_error {
        errors.push(json!({
            "kind": "Error",
            "message": message,
            "line": 0,
            "column": null
        }));
    }

    json!({
        "document": document,
        "errors": errors
    })
}

fn inject_default_styles(document_html: String) -> String {
    let styled_document = format!("<style>\n{}\n</style>\n", DEFAULT_STANDALONE_STYLES);

    if let Some(head_end) = document_html.find("</head>") {
        let mut html = document_html;
        html.insert_str(head_end, &styled_document);
        return wrap_body_in_main(html);
    }

    wrap_body_in_main(format!("{styled_document}{document_html}"))
}

fn wrap_body_in_main(document_html: String) -> String {
    if let Some(body_start) = document_html.find("<body>\n") {
        let mut html = document_html;
        let insert_at = body_start + "<body>\n".len();
        html.insert_str(insert_at, "<main>\n");

        if let Some(body_end) = html.rfind("\n</body>") {
            html.insert_str(body_end, "\n</main>");
        } else if let Some(body_end) = html.rfind("</body>") {
            html.insert_str(body_end, "</main>\n");
        }

        return html;
    }

    document_html
}

#[cfg(test)]
mod tests {
    use super::{parse_result, parse_to_json, render_html, render_html_standalone};
    use serde_json::Value;

    #[test]
    fn parse_to_json_returns_document_shape() {
        let json = parse_to_json("# Hello");
        let value: Value = serde_json::from_str(&json).expect("valid json");

        assert!(value.get("document").is_some());
        assert!(value.get("errors").is_some());
        assert_eq!(
            value["document"]["body"][0]["type"],
            Value::String("Heading".to_string())
        );
    }

    #[test]
    fn render_html_uses_partial_parse_result() {
        let result = parse_result("::aside\nMissing close");
        assert!(!result.errors.is_empty());

        let html = render_html("::aside\nMissing close");
        assert!(html.contains("data-directive=\"aside\""));
        assert!(html.contains("<p>Missing close</p>"));
    }

    #[test]
    fn render_html_standalone_embeds_styles() {
        let html = render_html_standalone("# Hello");

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("<main>"));
        assert!(html.contains("<h1>Hello</h1>"));
    }
}
