use crate::Frontmatter;
use std::collections::HashMap;

pub(crate) fn parse_frontmatter(input: &str) -> (Option<Frontmatter>, &str) {
    let Some((first_line, rest, opener_len)) = split_first_line(input) else {
        return (None, input);
    };

    if first_line != "---" {
        return (None, input);
    }

    let mut remaining = rest;
    let mut line_start = opener_len;

    while let Some((line, next_rest, line_len)) = split_first_line(remaining) {
        if line == "---" {
            let raw = input[opener_len..line_start].to_string();
            let fields = if raw.is_empty() {
                HashMap::new()
            } else {
                serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(&raw).unwrap_or_default()
            };

            return (Some(Frontmatter { raw, fields }), next_rest);
        }

        line_start += line_len;
        remaining = next_rest;
    }

    (None, input)
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
