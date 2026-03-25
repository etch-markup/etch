use crate::Inline;

use super::parse_inlines;

pub(super) fn try_parse_inline_directive(input: &str, index: usize) -> Option<(Inline, usize)> {
    if input.as_bytes().get(index).copied()? != b':' {
        return None;
    }

    let mut remainder = input.get(index + 1..)?;
    if !remainder
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }

    let name_len = parse_inline_directive_name_length(remainder)?;
    if name_len == 0 {
        return None;
    }

    let name = remainder[..name_len].to_string();
    remainder = &remainder[name_len..];

    let mut content = None;
    if remainder.starts_with('[') {
        let (content_text, next_remainder) =
            super::super::directive::parse_balanced_bracket_segment(remainder)?;
        content = Some(parse_inlines(content_text));
        remainder = next_remainder;
    }

    let mut attrs = None;
    if remainder.starts_with('{') {
        let (parsed_attrs, next_remainder) =
            super::super::attributes::parse_attributes_segment(remainder)?;
        attrs = Some(parsed_attrs);
        remainder = next_remainder;
    }

    Some((
        Inline::InlineDirective {
            name,
            content,
            attrs,
        },
        input.len() - remainder.len(),
    ))
}

fn parse_inline_directive_name_length(input: &str) -> Option<usize> {
    let mut length = 0;

    for (index, ch) in input.char_indices() {
        if ch.is_ascii_alphabetic() || ch == '-' {
            length = index + ch.len_utf8();
            continue;
        }

        if matches!(ch, '[' | '{') || is_inline_directive_boundary(ch) {
            return Some(length);
        }

        return None;
    }

    Some(length)
}

fn is_inline_directive_boundary(ch: char) -> bool {
    ch.is_whitespace() || !(ch.is_ascii_alphanumeric() || ch == '_')
}
