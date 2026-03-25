use crate::Inline;

use super::{
    parse_inlines,
    util::{char_after, char_before},
};

pub(super) fn try_parse_link(input: &str, index: usize) -> Option<(Inline, usize)> {
    if char_before(input, index) == Some('!') {
        return None;
    }

    let text_start = index + 1;
    let text_end = find_balanced_closing(input, text_start, '[', ']')?;
    let paren_start = text_end + 1;

    if input.as_bytes().get(paren_start).copied()? != b'(' {
        return None;
    }

    let destination_start = paren_start + 1;
    let destination_end = find_balanced_closing(input, destination_start, '(', ')')?;
    let (url, title) = parse_link_destination(&input[destination_start..destination_end])?;

    Some((
        Inline::Link {
            url,
            title,
            content: parse_inlines(&input[text_start..text_end]),
            attrs: None,
        },
        destination_end + 1,
    ))
}

pub(super) fn try_parse_footnote_reference(input: &str, index: usize) -> Option<(Inline, usize)> {
    if input.as_bytes().get(index).copied()? != b'['
        || input.as_bytes().get(index + 1).copied()? != b'^'
    {
        return None;
    }

    let label_start = index + 2;
    let label_end = input[label_start..].find(']')? + label_start;

    if label_end == label_start {
        return None;
    }

    Some((
        Inline::FootnoteReference {
            label: input[label_start..label_end].to_string(),
        },
        label_end + 1,
    ))
}

pub(super) fn try_parse_image(input: &str, index: usize) -> Option<(Inline, usize)> {
    if input.as_bytes().get(index).copied()? != b'!'
        || input.as_bytes().get(index + 1).copied()? != b'['
    {
        return None;
    }

    let alt_start = index + 2;
    let alt_end = find_balanced_closing(input, alt_start, '[', ']')?;
    let paren_start = alt_end + 1;

    if input.as_bytes().get(paren_start).copied()? != b'(' {
        return None;
    }

    let destination_start = paren_start + 1;
    let destination_end = find_balanced_closing(input, destination_start, '(', ')')?;
    let (url, title) = parse_link_destination(&input[destination_start..destination_end])?;
    let mut next_index = destination_end + 1;
    let mut attrs = None;

    if let Some((parsed_attrs, remainder)) =
        super::super::attributes::parse_attributes_segment(&input[next_index..])
    {
        attrs = Some(parsed_attrs);
        next_index = input.len() - remainder.len();
    }

    Some((
        Inline::Image {
            url,
            alt: input[alt_start..alt_end].to_string(),
            title,
            attrs,
        },
        next_index,
    ))
}

pub(super) fn try_parse_autolink(input: &str, index: usize) -> Option<(Inline, usize)> {
    let scheme_len = if input[index..].starts_with("https://") {
        "https://".len()
    } else if input[index..].starts_with("http://") {
        "http://".len()
    } else {
        return None;
    };

    let mut end = index + scheme_len;

    while end < input.len() {
        let ch = char_after(input, end)?;

        if ch.is_whitespace() {
            break;
        }

        end += ch.len_utf8();
    }

    if end == index + scheme_len {
        return None;
    }

    Some((
        Inline::AutoLink {
            url: input[index..end].to_string(),
        },
        end,
    ))
}

fn find_balanced_closing(input: &str, mut index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1;
    let mut escaped = false;

    while index < input.len() {
        let ch = char_after(input, index)?;

        if escaped {
            escaped = false;
            index += ch.len_utf8();
            continue;
        }

        match ch {
            '\\' => escaped = true,
            candidate if candidate == open => depth += 1,
            candidate if candidate == close => {
                depth -= 1;

                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }

        index += ch.len_utf8();
    }

    None
}

fn parse_link_destination(input: &str) -> Option<(String, Option<String>)> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return None;
    }

    let split_at = trimmed
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
        .unwrap_or(trimmed.len());

    let url = trimmed[..split_at].trim();

    if url.is_empty() {
        return None;
    }

    let remainder = trimmed[split_at..].trim();

    if remainder.is_empty() {
        return Some((url.to_string(), None));
    }

    let title = parse_quoted_link_title(remainder)?;
    Some((url.to_string(), Some(title)))
}

fn parse_quoted_link_title(input: &str) -> Option<String> {
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    let mut title = String::with_capacity(inner.len());
    let mut escaped = false;

    for ch in inner.chars() {
        if escaped {
            title.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        title.push(ch);
    }

    if escaped {
        title.push('\\');
    }

    Some(title)
}
