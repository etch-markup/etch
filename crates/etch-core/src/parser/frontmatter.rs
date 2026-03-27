use crate::{Frontmatter, FrontmatterValue, ParseError, ParseErrorKind};
use std::collections::BTreeMap;

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
                (BTreeMap::new(), Vec::new())
            } else {
                match parse_frontmatter_fields(&raw) {
                    Ok(fields) => (fields, Vec::new()),
                    Err(error) => (
                        BTreeMap::new(),
                        vec![ParseError {
                            kind: ParseErrorKind::Error,
                            message: format!(
                                "invalid frontmatter: {} on line {}",
                                error.message, error.line
                            ),
                            line: error.line,
                            column: Some(error.column),
                        }],
                    ),
                }
            };

            return (Some(Frontmatter { raw, fields }), next_rest, errors);
        }

        line_start += line_len;
        remaining = next_rest;
    }

    (None, input, Vec::new())
}

#[derive(Debug, Clone, PartialEq)]
struct FrontmatterParseError {
    message: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Copy)]
struct FrontmatterLine<'a> {
    number: usize,
    indent: usize,
    content: &'a str,
}

fn parse_frontmatter_fields(
    raw: &str,
) -> Result<BTreeMap<String, FrontmatterValue>, FrontmatterParseError> {
    let lines = raw
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if line.trim().is_empty() {
                return None;
            }

            let indent = line.chars().take_while(|ch| *ch == ' ').count();

            Some(FrontmatterLine {
                number: index + 2,
                indent,
                content: &line[indent..],
            })
        })
        .collect::<Vec<_>>();

    let mut index = 0;
    let fields = parse_mapping(&lines, &mut index, 0)?;

    if let Some(line) = lines.get(index) {
        return Err(FrontmatterParseError {
            message: "unexpected indentation".to_string(),
            line: line.number,
            column: line.indent + 1,
        });
    }

    Ok(fields)
}

fn parse_mapping(
    lines: &[FrontmatterLine<'_>],
    index: &mut usize,
    indent: usize,
) -> Result<BTreeMap<String, FrontmatterValue>, FrontmatterParseError> {
    let mut values = BTreeMap::new();

    while let Some(line) = lines.get(*index) {
        if line.indent < indent {
            break;
        }

        if line.indent > indent {
            return Err(FrontmatterParseError {
                message: "unexpected indentation".to_string(),
                line: line.number,
                column: line.indent + 1,
            });
        }

        let Some((key, remainder, key_width)) = split_key_value(line.content) else {
            return Err(FrontmatterParseError {
                message: "expected `key: value`".to_string(),
                line: line.number,
                column: 1,
            });
        };

        *index += 1;

        let value = if let Some(value_text) = remainder {
            parse_value(value_text, line.number, line.indent + key_width + 2)?
        } else if let Some(next_line) = lines.get(*index) {
            if next_line.indent > indent {
                parse_mapping(lines, index, next_line.indent)?.into()
            } else {
                FrontmatterValue::Null
            }
        } else {
            FrontmatterValue::Null
        };

        values.insert(key.to_string(), value);
    }

    Ok(values)
}

fn split_key_value(line: &str) -> Option<(&str, Option<&str>, usize)> {
    let (key, value) = line.split_once(':')?;
    let key = key.trim();

    if key.is_empty() {
        return None;
    }

    let value = value.trim_start();
    let value = if value.is_empty() { None } else { Some(value) };

    Some((key, value, key.len()))
}

fn parse_value(
    input: &str,
    line: usize,
    column: usize,
) -> Result<FrontmatterValue, FrontmatterParseError> {
    if input.starts_with('"') {
        let (value, consumed) = parse_quoted_string(input, line, column)?;
        ensure_only_trailing_whitespace(&input[consumed..], line, column + consumed)?;
        return Ok(FrontmatterValue::String(value));
    }

    if input.starts_with('[') {
        let (value, consumed) = parse_array(input, line, column)?;
        ensure_only_trailing_whitespace(&input[consumed..], line, column + consumed)?;
        return Ok(value);
    }

    Ok(parse_scalar(input))
}

fn ensure_only_trailing_whitespace(
    input: &str,
    line: usize,
    column: usize,
) -> Result<(), FrontmatterParseError> {
    if input.trim().is_empty() {
        return Ok(());
    }

    Err(FrontmatterParseError {
        message: "unexpected trailing content".to_string(),
        line,
        column,
    })
}

fn parse_quoted_string(
    input: &str,
    line: usize,
    column: usize,
) -> Result<(String, usize), FrontmatterParseError> {
    let mut value = String::new();
    let mut escaped = false;

    for (index, ch) in input.char_indices().skip(1) {
        if escaped {
            let translated = match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            };
            value.push(translated);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Ok((value, index + ch.len_utf8())),
            _ => value.push(ch),
        }
    }

    Err(FrontmatterParseError {
        message: "unterminated quoted string".to_string(),
        line,
        column,
    })
}

fn parse_array(
    input: &str,
    line: usize,
    column: usize,
) -> Result<(FrontmatterValue, usize), FrontmatterParseError> {
    let bytes = input.as_bytes();
    let mut index = 1;
    let mut values = Vec::new();

    loop {
        while let Some(byte) = bytes.get(index) {
            if byte.is_ascii_whitespace() {
                index += 1;
            } else {
                break;
            }
        }

        match bytes.get(index) {
            Some(b']') => return Ok((FrontmatterValue::Array(values), index + 1)),
            Some(_) => {
                let (value, consumed) = parse_inline_value(&input[index..], line, column + index)?;
                values.push(value);
                index += consumed;
            }
            None => {
                return Err(FrontmatterParseError {
                    message: "unterminated array".to_string(),
                    line,
                    column,
                });
            }
        }

        while let Some(byte) = bytes.get(index) {
            if byte.is_ascii_whitespace() {
                index += 1;
            } else {
                break;
            }
        }

        match bytes.get(index) {
            Some(b',') => index += 1,
            Some(b']') => return Ok((FrontmatterValue::Array(values), index + 1)),
            Some(_) => {
                return Err(FrontmatterParseError {
                    message: "expected `,` or `]` in array".to_string(),
                    line,
                    column: column + index,
                });
            }
            None => {
                return Err(FrontmatterParseError {
                    message: "unterminated array".to_string(),
                    line,
                    column,
                });
            }
        }
    }
}

fn parse_inline_value(
    input: &str,
    line: usize,
    column: usize,
) -> Result<(FrontmatterValue, usize), FrontmatterParseError> {
    if input.starts_with('"') {
        let (value, consumed) = parse_quoted_string(input, line, column)?;
        return Ok((FrontmatterValue::String(value), consumed));
    }

    if input.starts_with('[') {
        return parse_array(input, line, column);
    }

    let consumed = input
        .find(|ch: char| ch == ',' || ch == ']')
        .unwrap_or(input.len());
    let value = input[..consumed].trim_end();

    if value.is_empty() {
        return Err(FrontmatterParseError {
            message: "expected array value".to_string(),
            line,
            column,
        });
    }

    Ok((parse_scalar(value), consumed))
}

fn parse_scalar(input: &str) -> FrontmatterValue {
    let value = input.trim();

    match value {
        "true" => FrontmatterValue::Bool(true),
        "false" => FrontmatterValue::Bool(false),
        "null" | "~" => FrontmatterValue::Null,
        _ => {
            if let Ok(number) = value.parse::<i64>() {
                return FrontmatterValue::Integer(number);
            }

            if is_float_literal(value) {
                if let Ok(number) = value.parse::<f64>() {
                    return FrontmatterValue::Float(number);
                }
            }

            FrontmatterValue::String(value.to_string())
        }
    }
}

fn is_float_literal(value: &str) -> bool {
    value.contains('.') || value.contains('e') || value.contains('E')
}

impl From<BTreeMap<String, FrontmatterValue>> for FrontmatterValue {
    fn from(value: BTreeMap<String, FrontmatterValue>) -> Self {
        Self::Object(value)
    }
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
