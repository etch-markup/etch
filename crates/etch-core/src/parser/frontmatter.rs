use crate::{Frontmatter, ParseError, ParseErrorKind};
use std::collections::HashMap;

pub(crate) fn parse_frontmatter(input: &str) -> (Option<Frontmatter>, &str, Vec<ParseError>) {
    let Some((first_line, rest, opener_len)) = split_first_line(input) else {
        return (None, input, Vec::new());
    };

    if first_line != "---" {
        return (None, input, Vec::new());
    }

    let mut remaining = rest;
    let mut line_start = opener_len;

    while let Some((line, next_rest, line_len)) = split_first_line(remaining) {
        if line == "---" {
            let raw = input[opener_len..line_start].to_string();
            let (fields, errors) = if raw.is_empty() {
                (HashMap::new(), Vec::new())
            } else {
                match serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(&raw) {
                    Ok(fields) => (fields, Vec::new()),
                    Err(error) => {
                        let relative_line = error
                            .location()
                            .map(|location| location.line())
                            .unwrap_or(1);
                        let line = relative_line + 1;
                        let column = error.location().map(|location| location.column());
                        let summary = error
                            .to_string()
                            .replace('\n', " ")
                            .split_once(" at line ")
                            .map(|(message, _)| message.to_string())
                            .unwrap_or_else(|| error.to_string());

                        (
                            HashMap::new(),
                            vec![ParseError {
                                kind: ParseErrorKind::Error,
                                message: format!(
                                    "invalid frontmatter YAML: {} on line {}",
                                    summary, line
                                ),
                                line,
                                column,
                            }],
                        )
                    }
                }
            };

            return (Some(Frontmatter { raw, fields }), next_rest, errors);
        }

        line_start += line_len;
        remaining = next_rest;
    }

    (None, input, Vec::new())
}

pub(crate) fn split_first_line(input: &str) -> Option<(&str, &str, usize)> {
    if input.is_empty() {
        return None;
    }

    if let Some(newline_index) = input.find('\n') {
        let line = input[..newline_index]
            .strip_suffix('\r')
            .unwrap_or(&input[..newline_index]);
        return Some((line, &input[newline_index + 1..], newline_index + 1));
    }

    Some((input.strip_suffix('\r').unwrap_or(input), "", input.len()))
}
