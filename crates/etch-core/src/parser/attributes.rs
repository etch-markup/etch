use crate::Attributes;
use std::collections::HashMap;

pub(crate) fn parse_attributes_segment(input: &str) -> Option<(Attributes, &str)> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            if ch != '{' {
                return None;
            }

            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => in_quotes = !in_quotes,
            '}' if !in_quotes => {
                let attrs = parse_attributes_content(&input[1..index])?;
                return Some((attrs, &input[index + 1..]));
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn parse_attributes_content(input: &str) -> Option<Attributes> {
    let mut attrs = Attributes {
        id: None,
        classes: Vec::new(),
        pairs: HashMap::new(),
    };

    for token in split_attribute_tokens(input) {
        if token.is_empty() {
            continue;
        }

        if let Some(id) = token.strip_prefix('#') {
            if id.is_empty() {
                return None;
            }

            attrs.id = Some(id.to_string());
            continue;
        }

        if let Some(class) = token.strip_prefix('.') {
            if class.is_empty() {
                return None;
            }

            attrs.classes.push(class.to_string());
            continue;
        }

        let (key, value) = token.split_once('=')?;
        if key.is_empty() {
            return None;
        }

        let value = if value.starts_with('"') {
            let quoted = value.strip_prefix('"')?.strip_suffix('"')?;
            unescape_quoted_attribute_value(quoted)
        } else {
            value.to_string()
        };

        attrs.pairs.insert(key.to_string(), value);
    }

    Some(attrs)
}

pub(crate) fn split_attribute_tokens(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => {
                in_quotes = !in_quotes;
                start.get_or_insert(index);
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if let Some(token_start) = start.take() {
                    tokens.push(&input[token_start..index]);
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if let Some(token_start) = start {
        tokens.push(&input[token_start..]);
    }

    tokens
}

pub(crate) fn unescape_quoted_attribute_value(value: &str) -> String {
    let mut unescaped = String::with_capacity(value.len());
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            unescaped.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        unescaped.push(ch);
    }

    if escaped {
        unescaped.push('\\');
    }

    unescaped
}
