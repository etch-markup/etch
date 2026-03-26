use crate::Attributes;
use std::collections::HashMap;
use std::iter::Peekable;

pub(crate) fn parse_attributes_segment(input: &str) -> Option<(Attributes, &str)> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            if ch != '{' {
                return None;
            }

            let next = input[1..].chars().next()?;
            if !(next == '#' || next == '.' || next.is_ascii_alphabetic()) {
                return None;
            }

            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes && input[index + 1..].starts_with('"') => escaped = true,
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
            '\\' if in_quotes && input[index + 1..].starts_with('"') => escaped = true,
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
            if ch != '"' {
                unescaped.push('\\');
            }
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

pub(crate) fn split_trailing_block_attributes(input: &str) -> Option<(&str, Attributes)> {
    for (index, ch) in input.char_indices().rev() {
        if ch != '{' || index == 0 {
            continue;
        }

        if !input[..index]
            .chars()
            .next_back()
            .is_some_and(|previous| previous.is_whitespace())
        {
            continue;
        }

        if let Some((attrs, remainder)) = parse_attributes_segment(&input[index..]) {
            if remainder.trim().is_empty() {
                return Some((input[..index].trim_end(), attrs));
            }
        }
    }

    None
}

pub(crate) fn parse_attribute_only_line(input: &str) -> Option<Attributes> {
    let trimmed = input.trim();
    let (attrs, remainder) = parse_attributes_segment(trimmed)?;
    remainder.trim().is_empty().then_some(attrs)
}

pub(crate) fn take_attribute_only_line<'a, I>(lines: &mut Peekable<I>) -> Option<Attributes>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let attrs = parse_attribute_only_line(lines.peek()?.1)?;
    lines.next();
    Some(attrs)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_attribute_only_line, parse_attributes_segment, split_trailing_block_attributes,
        unescape_quoted_attribute_value,
    };
    use crate::Attributes;
    use std::collections::HashMap;

    #[test]
    fn parses_attribute_segments_only_when_the_first_character_is_valid() {
        assert!(parse_attributes_segment("{#title}").is_some());
        assert!(parse_attributes_segment("{.hero}").is_some());
        assert!(parse_attributes_segment("{lang=en}").is_some());
        assert!(parse_attributes_segment("{~ comment ~}").is_none());
        assert!(parse_attributes_segment("{1=2}").is_none());
    }

    #[test]
    fn unescapes_only_escaped_quotes_in_quoted_values() {
        assert_eq!(
            unescape_quoted_attribute_value(r#"value with \"quotes\" and \slashes"#),
            "value with \"quotes\" and \\slashes"
        );
    }

    #[test]
    fn splits_trailing_block_attributes_only_when_separated_by_whitespace() {
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("lang".to_string(), "en".to_string());

        assert_eq!(
            split_trailing_block_attributes("Title {#title .hero lang=en}"),
            Some((
                "Title",
                Attributes {
                    id: Some("title".to_string()),
                    classes: vec!["hero".to_string()],
                    pairs: expected_pairs,
                }
            ))
        );
        assert!(split_trailing_block_attributes("![alt](img.png){width=80%}").is_none());
    }

    #[test]
    fn parses_attribute_only_lines() {
        let attrs = parse_attribute_only_line("  {.wide key=\"quoted value\"}  ")
            .expect("expected attributes");

        assert_eq!(attrs.id, None);
        assert_eq!(attrs.classes, vec!["wide".to_string()]);
        assert_eq!(attrs.pairs.get("key"), Some(&"quoted value".to_string()));
    }
}
