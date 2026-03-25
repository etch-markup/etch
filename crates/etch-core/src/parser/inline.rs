use crate::Inline;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StarDelimiter {
    Emphasis,
    Strong,
    StrongEmphasis,
}

impl StarDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Emphasis => 1,
            Self::Strong => 2,
            Self::StrongEmphasis => 3,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Emphasis => Inline::Emphasis { content },
            Self::Strong => Inline::Strong { content },
            Self::StrongEmphasis => Inline::Strong {
                content: vec![Inline::Emphasis { content }],
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TildeDelimiter {
    Subscript,
    Strikethrough,
}

impl TildeDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Subscript => 1,
            Self::Strikethrough => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Subscript => Inline::Subscript { content },
            Self::Strikethrough => Inline::Strikethrough { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaretDelimiter {
    Superscript,
}

impl CaretDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Superscript => 1,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Superscript => Inline::Superscript { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EqualDelimiter {
    Highlight,
}

impl EqualDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Highlight => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Highlight => Inline::Highlight { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlusDelimiter {
    Insert,
}

impl PlusDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Insert => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Insert => Inline::Insert { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Delimiter {
    Star(StarDelimiter),
    Tilde(TildeDelimiter),
    Caret(CaretDelimiter),
    Equal(EqualDelimiter),
    Plus(PlusDelimiter),
}

impl Delimiter {
    fn len(self) -> usize {
        match self {
            Self::Star(delimiter) => delimiter.len(),
            Self::Tilde(delimiter) => delimiter.len(),
            Self::Caret(delimiter) => delimiter.len(),
            Self::Equal(delimiter) => delimiter.len(),
            Self::Plus(delimiter) => delimiter.len(),
        }
    }

    fn marker(self) -> u8 {
        match self {
            Self::Star(_) => b'*',
            Self::Tilde(_) => b'~',
            Self::Caret(_) => b'^',
            Self::Equal(_) => b'=',
            Self::Plus(_) => b'+',
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Star(delimiter) => delimiter.wrap(content),
            Self::Tilde(delimiter) => delimiter.wrap(content),
            Self::Caret(delimiter) => delimiter.wrap(content),
            Self::Equal(delimiter) => delimiter.wrap(content),
            Self::Plus(delimiter) => delimiter.wrap(content),
        }
    }

    fn matches_run(self, run_len: usize) -> bool {
        match self {
            Self::Star(delimiter) => run_len >= delimiter.len(),
            Self::Tilde(delimiter) => run_len == delimiter.len(),
            Self::Caret(delimiter) => run_len == delimiter.len(),
            Self::Equal(delimiter) => run_len == delimiter.len(),
            Self::Plus(delimiter) => run_len == delimiter.len(),
        }
    }
}

struct ParseResult {
    nodes: Vec<Inline>,
    next_index: usize,
    closed: bool,
}

#[allow(dead_code)]
pub(crate) fn parse_inlines(input: &str) -> Vec<Inline> {
    parse_segment(input, 0, None).nodes
}

fn parse_segment(input: &str, mut index: usize, stop: Option<Delimiter>) -> ParseResult {
    let mut nodes = Vec::new();
    let mut text_start = index;

    while index < input.len() {
        if let Some(delimiter) = stop {
            if can_close(
                input,
                index,
                delimiter,
                nodes.is_empty() && text_start == index,
            ) {
                push_text(&mut nodes, &input[text_start..index]);

                return ParseResult {
                    nodes,
                    next_index: index + delimiter.len(),
                    closed: true,
                };
            }
        }

        let byte = input.as_bytes()[index];

        if byte == b'\\' {
            if let Some(next_index) = try_parse_hard_break(input, index) {
                push_text(&mut nodes, &input[text_start..index]);
                nodes.push(Inline::HardBreak);
                index = next_index;
                text_start = index;
                continue;
            }

            if let Some(next_index) = try_parse_escaped_literal(input, index) {
                push_text(&mut nodes, &input[text_start..index]);
                push_text(&mut nodes, &input[index + 1..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }
        }

        if let Some(next_index) = try_parse_soft_break(input, index) {
            push_text(&mut nodes, &input[text_start..index]);
            nodes.push(Inline::SoftBreak);
            index = next_index;
            text_start = index;
            continue;
        }

        if byte == b'h' {
            if let Some((inline, next_index)) = try_parse_autolink(input, index) {
                push_text(&mut nodes, &input[text_start..index]);
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }
        }

        if byte == b'!' && input.as_bytes().get(index + 1).copied() == Some(b'[') {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_image(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        if byte == b'[' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_footnote_reference(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            if let Some((inline, next_index)) = try_parse_link(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        if byte == b':' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_inline_directive(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        if byte == b'`' {
            push_text(&mut nodes, &input[text_start..index]);
            let run_len = count_delimiters(input, index, b'`');

            if let Some((inline, next_index)) = try_parse_inline_code(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + run_len]);
            index += run_len;
            text_start = index;
            continue;
        }

        if byte == b'*' || byte == b'~' || byte == b'^' || byte == b'=' || byte == b'+' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_delimiter_run(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        index += next_char_len(input, index);
    }

    push_text(&mut nodes, &input[text_start..index]);

    ParseResult {
        nodes,
        next_index: index,
        closed: false,
    }
}

fn try_parse_delimiter_run(input: &str, index: usize) -> Option<(Inline, usize)> {
    let delimiter = parse_delimiter(input, index)?;

    if !can_open(input, index, delimiter) {
        return None;
    }

    let inner = parse_segment(input, index + delimiter.len(), Some(delimiter));

    if inner.closed && !inner.nodes.is_empty() {
        return Some((delimiter.wrap(inner.nodes), inner.next_index));
    }

    None
}

fn try_parse_hard_break(input: &str, index: usize) -> Option<usize> {
    let remainder = input.get(index..)?;

    if remainder.starts_with("\\\r\n") {
        return Some(index + "\\\r\n".len());
    }

    remainder
        .starts_with("\\\n")
        .then_some(index + "\\\n".len())
}

fn try_parse_escaped_literal(input: &str, index: usize) -> Option<usize> {
    let escaped = char_after(input, index + 1)?;

    matches!(escaped, '*' | '~' | '^' | '=' | '+' | '[' | ']' | '\\')
        .then_some(index + 1 + escaped.len_utf8())
}

fn try_parse_soft_break(input: &str, index: usize) -> Option<usize> {
    let remainder = input.get(index..)?;

    if remainder.starts_with("\r\n") {
        return Some(index + "\r\n".len());
    }

    if remainder.starts_with('\n') || remainder.starts_with('\r') {
        return Some(index + 1);
    }

    None
}

fn try_parse_link(input: &str, index: usize) -> Option<(Inline, usize)> {
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

fn try_parse_footnote_reference(input: &str, index: usize) -> Option<(Inline, usize)> {
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

fn try_parse_image(input: &str, index: usize) -> Option<(Inline, usize)> {
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
        super::attributes::parse_attributes_segment(&input[next_index..])
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

fn try_parse_inline_directive(input: &str, index: usize) -> Option<(Inline, usize)> {
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
            super::directive::parse_balanced_bracket_segment(remainder)?;
        content = Some(parse_inlines(content_text));
        remainder = next_remainder;
    }

    let mut attrs = None;
    if remainder.starts_with('{') {
        let (parsed_attrs, next_remainder) =
            super::attributes::parse_attributes_segment(remainder)?;
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

fn try_parse_autolink(input: &str, index: usize) -> Option<(Inline, usize)> {
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

fn try_parse_inline_code(input: &str, index: usize) -> Option<(Inline, usize)> {
    let delimiter_len = count_delimiters(input, index, b'`');

    if !matches!(delimiter_len, 1 | 2) {
        return None;
    }

    let content_start = index + delimiter_len;
    let closing_index = find_closing_backticks(input, content_start, delimiter_len)?;

    Some((
        Inline::InlineCode {
            value: input[content_start..closing_index].to_string(),
        },
        closing_index + delimiter_len,
    ))
}

fn parse_delimiter(input: &str, index: usize) -> Option<Delimiter> {
    let byte = input.as_bytes().get(index).copied()?;

    match byte {
        b'*' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Star(StarDelimiter::Emphasis)),
            2 => Some(Delimiter::Star(StarDelimiter::Strong)),
            3 => Some(Delimiter::Star(StarDelimiter::StrongEmphasis)),
            _ => None,
        },
        b'~' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Tilde(TildeDelimiter::Subscript)),
            2 => Some(Delimiter::Tilde(TildeDelimiter::Strikethrough)),
            _ => None,
        },
        b'^' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Caret(CaretDelimiter::Superscript)),
            _ => None,
        },
        b'=' => match count_delimiters(input, index, byte) {
            2 => Some(Delimiter::Equal(EqualDelimiter::Highlight)),
            _ => None,
        },
        b'+' => match count_delimiters(input, index, byte) {
            2 => Some(Delimiter::Plus(PlusDelimiter::Insert)),
            _ => None,
        },
        _ => None,
    }
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

fn can_open(input: &str, index: usize, delimiter: Delimiter) -> bool {
    char_after(input, index + delimiter.len()).is_some_and(|ch| !ch.is_whitespace())
}

fn can_close(input: &str, index: usize, delimiter: Delimiter, empty_content: bool) -> bool {
    !empty_content
        && delimiter.matches_run(count_delimiters(input, index, delimiter.marker()))
        && char_before(input, index).is_some_and(|ch| !ch.is_whitespace())
}

fn count_delimiters(input: &str, index: usize, byte: u8) -> usize {
    input[index..]
        .bytes()
        .take_while(|candidate| *candidate == byte)
        .count()
}

fn find_closing_backticks(input: &str, mut index: usize, delimiter_len: usize) -> Option<usize> {
    while index < input.len() {
        if input.as_bytes()[index] == b'`' && count_delimiters(input, index, b'`') >= delimiter_len
        {
            return Some(index);
        }

        index += next_char_len(input, index);
    }

    None
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

fn next_char_len(input: &str, index: usize) -> usize {
    input[index..]
        .chars()
        .next()
        .expect("index points to a valid character boundary")
        .len_utf8()
}

fn char_after(input: &str, index: usize) -> Option<char> {
    input.get(index..)?.chars().next()
}

fn char_before(input: &str, index: usize) -> Option<char> {
    input.get(..index)?.chars().next_back()
}

fn push_text(nodes: &mut Vec<Inline>, value: &str) {
    if value.is_empty() {
        return;
    }

    if let Some(Inline::Text { value: existing }) = nodes.last_mut() {
        existing.push_str(value);
        return;
    }

    nodes.push(Inline::Text {
        value: value.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::parse_inlines;
    use crate::{Attributes, Inline};
    use std::collections::HashMap;

    #[test]
    fn parses_emphasis() {
        assert_eq!(
            parse_inlines("before *italic* after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Emphasis {
                    content: vec![Inline::Text {
                        value: "italic".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_strong() {
        assert_eq!(
            parse_inlines("before **bold** after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Strong {
                    content: vec![Inline::Text {
                        value: "bold".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_strong_emphasis_as_strong_wrapping_emphasis() {
        assert_eq!(
            parse_inlines("***both***"),
            vec![Inline::Strong {
                content: vec![Inline::Emphasis {
                    content: vec![Inline::Text {
                        value: "both".to_string(),
                    }],
                }],
            }]
        );
    }

    #[test]
    fn parses_nested_star_delimiters() {
        assert_eq!(
            parse_inlines("**bold with *italic* inside**"),
            vec![Inline::Strong {
                content: vec![
                    Inline::Text {
                        value: "bold with ".to_string(),
                    },
                    Inline::Emphasis {
                        content: vec![Inline::Text {
                            value: "italic".to_string(),
                        }],
                    },
                    Inline::Text {
                        value: " inside".to_string(),
                    },
                ],
            }]
        );

        assert_eq!(
            parse_inlines("*italic with **bold** inside*"),
            vec![Inline::Emphasis {
                content: vec![
                    Inline::Text {
                        value: "italic with ".to_string(),
                    },
                    Inline::Strong {
                        content: vec![Inline::Text {
                            value: "bold".to_string(),
                        }],
                    },
                    Inline::Text {
                        value: " inside".to_string(),
                    },
                ],
            }]
        );
    }

    #[test]
    fn parses_strikethrough() {
        assert_eq!(
            parse_inlines("before ~~struck~~ after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Strikethrough {
                    content: vec![Inline::Text {
                        value: "struck".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn distinguishes_strikethrough_from_subscript() {
        assert_eq!(
            parse_inlines("~~struck~~ and ~sub~"),
            vec![
                Inline::Strikethrough {
                    content: vec![Inline::Text {
                        value: "struck".to_string(),
                    }],
                },
                Inline::Text {
                    value: " and ".to_string(),
                },
                Inline::Subscript {
                    content: vec![Inline::Text {
                        value: "sub".to_string(),
                    }],
                },
            ]
        );
    }

    #[test]
    fn parses_superscript() {
        assert_eq!(
            parse_inlines("before ^super^ after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Superscript {
                    content: vec![Inline::Text {
                        value: "super".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_highlight() {
        assert_eq!(
            parse_inlines("before ==marked== after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Highlight {
                    content: vec![Inline::Text {
                        value: "marked".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_insert() {
        assert_eq!(
            parse_inlines("before ++added++ after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::Insert {
                    content: vec![Inline::Text {
                        value: "added".to_string(),
                    }],
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn leaves_invalid_caret_runs_literal() {
        assert_eq!(
            parse_inlines("^^ double carets ^^"),
            vec![Inline::Text {
                value: "^^ double carets ^^".to_string(),
            }]
        );
    }

    #[test]
    fn parses_inline_code_spans() {
        assert_eq!(
            parse_inlines("`printf()` opens this sentence and `npm test` runs later"),
            vec![
                Inline::InlineCode {
                    value: "printf()".to_string(),
                },
                Inline::Text {
                    value: " opens this sentence and ".to_string(),
                },
                Inline::InlineCode {
                    value: "npm test".to_string(),
                },
                Inline::Text {
                    value: " runs later".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_double_backtick_code_spans() {
        assert_eq!(
            parse_inlines("before ``code containing `backticks` inside`` after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::InlineCode {
                    value: "code containing `backticks` inside".to_string(),
                },
                Inline::Text {
                    value: " after".to_string(),
                },
            ]
        );
    }

    #[test]
    fn treats_code_span_content_as_raw_text() {
        assert_eq!(
            parse_inlines("`*not italic* and **not bold**`"),
            vec![Inline::InlineCode {
                value: "*not italic* and **not bold**".to_string(),
            }]
        );
    }

    #[test]
    fn does_not_escape_backticks_inside_code_spans() {
        assert_eq!(
            parse_inlines("`a\\`b`"),
            vec![
                Inline::InlineCode {
                    value: "a\\".to_string(),
                },
                Inline::Text {
                    value: "b`".to_string(),
                },
            ]
        );
    }

    #[test]
    fn leaves_unclosed_backtick_runs_literal() {
        assert_eq!(
            parse_inlines("before ``code after"),
            vec![Inline::Text {
                value: "before ``code after".to_string(),
            }]
        );
    }

    #[test]
    fn leaves_empty_or_whitespace_delimiters_literal() {
        assert_eq!(
            parse_inlines("** **** * text* *text * == ++++ ++ text++ ++text ++"),
            vec![Inline::Text {
                value: "** **** * text* *text * == ++++ ++ text++ ++text ++".to_string(),
            }]
        );
    }

    #[test]
    fn parses_basic_links() {
        assert_eq!(
            parse_inlines("Read the [Etch guide](https://docs.etch-lang.dev/guide)."),
            vec![
                Inline::Text {
                    value: "Read the ".to_string(),
                },
                Inline::Link {
                    url: "https://docs.etch-lang.dev/guide".to_string(),
                    title: None,
                    content: vec![Inline::Text {
                        value: "Etch guide".to_string(),
                    }],
                    attrs: None,
                },
                Inline::Text {
                    value: ".".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_links_with_titles() {
        assert_eq!(
            parse_inlines("[reference](https://docs.etch-lang.dev \"Core syntax reference\")"),
            vec![Inline::Link {
                url: "https://docs.etch-lang.dev".to_string(),
                title: Some("Core syntax reference".to_string()),
                content: vec![Inline::Text {
                    value: "reference".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parses_links_with_formatted_text() {
        assert_eq!(
            parse_inlines("[**Etch** *quickstart*](https://docs.etch-lang.dev/guide)"),
            vec![Inline::Link {
                url: "https://docs.etch-lang.dev/guide".to_string(),
                title: None,
                content: vec![
                    Inline::Strong {
                        content: vec![Inline::Text {
                            value: "Etch".to_string(),
                        }],
                    },
                    Inline::Text {
                        value: " ".to_string(),
                    },
                    Inline::Emphasis {
                        content: vec![Inline::Text {
                            value: "quickstart".to_string(),
                        }],
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parses_basic_images() {
        assert_eq!(
            parse_inlines("![alt](https://docs.etch-lang.dev/image.png)"),
            vec![Inline::Image {
                url: "https://docs.etch-lang.dev/image.png".to_string(),
                alt: "alt".to_string(),
                title: None,
                attrs: None,
            }]
        );
    }

    #[test]
    fn parses_images_with_titles() {
        assert_eq!(
            parse_inlines("![alt](https://docs.etch-lang.dev/image.png \"Camp marker\")"),
            vec![Inline::Image {
                url: "https://docs.etch-lang.dev/image.png".to_string(),
                alt: "alt".to_string(),
                title: Some("Camp marker".to_string()),
                attrs: None,
            }]
        );
    }

    #[test]
    fn parses_images_with_attributes() {
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("width".to_string(), "80%".to_string());

        assert_eq!(
            parse_inlines("![alt](https://docs.etch-lang.dev/image.png){width=80% .rounded}"),
            vec![Inline::Image {
                url: "https://docs.etch-lang.dev/image.png".to_string(),
                alt: "alt".to_string(),
                title: None,
                attrs: Some(Attributes {
                    id: None,
                    classes: vec!["rounded".to_string()],
                    pairs: expected_pairs,
                }),
            }]
        );
    }

    #[test]
    fn parses_footnote_references() {
        assert_eq!(
            parse_inlines("See [^guide] later"),
            vec![
                Inline::Text {
                    value: "See ".to_string(),
                },
                Inline::FootnoteReference {
                    label: "guide".to_string(),
                },
                Inline::Text {
                    value: " later".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_http_and_https_autolinks() {
        assert_eq!(
            parse_inlines("Visit https://example.com/path and http://docs.etch-lang.dev/guide"),
            vec![
                Inline::Text {
                    value: "Visit ".to_string(),
                },
                Inline::AutoLink {
                    url: "https://example.com/path".to_string(),
                },
                Inline::Text {
                    value: " and ".to_string(),
                },
                Inline::AutoLink {
                    url: "http://docs.etch-lang.dev/guide".to_string(),
                },
            ]
        );
    }

    #[test]
    fn autolinks_stop_at_whitespace_or_end_of_line() {
        assert_eq!(
            parse_inlines("See https://example.com/path\nThen read more"),
            vec![
                Inline::Text {
                    value: "See ".to_string(),
                },
                Inline::AutoLink {
                    url: "https://example.com/path".to_string(),
                },
                Inline::SoftBreak,
                Inline::Text {
                    value: "Then read more".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_soft_breaks_from_newlines() {
        assert_eq!(
            parse_inlines("First line\nSecond line\r\nThird line"),
            vec![
                Inline::Text {
                    value: "First line".to_string(),
                },
                Inline::SoftBreak,
                Inline::Text {
                    value: "Second line".to_string(),
                },
                Inline::SoftBreak,
                Inline::Text {
                    value: "Third line".to_string(),
                },
            ]
        );
    }

    #[test]
    fn leaves_bare_domains_and_non_http_schemes_literal() {
        assert_eq!(
            parse_inlines("example.com ftp://files.etch-lang.dev/releases/latest.zip"),
            vec![Inline::Text {
                value: "example.com ftp://files.etch-lang.dev/releases/latest.zip".to_string(),
            }]
        );
    }

    #[test]
    fn parses_hard_breaks_from_trailing_backslashes() {
        assert_eq!(
            parse_inlines("123 Main Street\\\nApartment 4B"),
            vec![
                Inline::Text {
                    value: "123 Main Street".to_string(),
                },
                Inline::HardBreak,
                Inline::Text {
                    value: "Apartment 4B".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_hard_breaks_inside_formatted_content() {
        assert_eq!(
            parse_inlines("*Starts here\\\nand ends here*"),
            vec![Inline::Emphasis {
                content: vec![
                    Inline::Text {
                        value: "Starts here".to_string(),
                    },
                    Inline::HardBreak,
                    Inline::Text {
                        value: "and ends here".to_string(),
                    },
                ],
            }]
        );
    }

    #[test]
    fn does_not_treat_two_trailing_spaces_as_hard_breaks() {
        assert_eq!(
            parse_inlines("First line  \nSecond line"),
            vec![
                Inline::Text {
                    value: "First line  ".to_string(),
                },
                Inline::SoftBreak,
                Inline::Text {
                    value: "Second line".to_string(),
                },
            ]
        );
    }

    #[test]
    fn leaves_non_trailing_backslashes_literal() {
        assert_eq!(
            parse_inlines("Path \\ server"),
            vec![Inline::Text {
                value: "Path \\ server".to_string(),
            }]
        );
    }

    #[test]
    fn parses_inline_directives_with_content_and_attributes() {
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("species".to_string(), "fox".to_string());
        expected_pairs.insert("mood".to_string(), "playful".to_string());

        assert_eq!(
            parse_inlines("I met :character[Sable]{species=fox mood=playful} nearby."),
            vec![
                Inline::Text {
                    value: "I met ".to_string(),
                },
                Inline::InlineDirective {
                    name: "character".to_string(),
                    content: Some(vec![Inline::Text {
                        value: "Sable".to_string(),
                    }]),
                    attrs: Some(Attributes {
                        id: None,
                        classes: Vec::new(),
                        pairs: expected_pairs,
                    }),
                },
                Inline::Text {
                    value: " nearby.".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_bare_inline_directives() {
        assert_eq!(
            parse_inlines("Insert :pagebreak here.\n:toc"),
            vec![
                Inline::Text {
                    value: "Insert ".to_string(),
                },
                Inline::InlineDirective {
                    name: "pagebreak".to_string(),
                    content: None,
                    attrs: None,
                },
                Inline::Text {
                    value: " here.".to_string(),
                },
                Inline::SoftBreak,
                Inline::InlineDirective {
                    name: "toc".to_string(),
                    content: None,
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parses_inline_directive_content_with_formatting_and_escaped_brackets() {
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("text".to_string(), "extra info".to_string());

        assert_eq!(
            parse_inlines(":tooltip[**bold** and use \\] plus \\[ too]{text=\"extra info\"}"),
            vec![Inline::InlineDirective {
                name: "tooltip".to_string(),
                content: Some(vec![
                    Inline::Strong {
                        content: vec![Inline::Text {
                            value: "bold".to_string(),
                        }],
                    },
                    Inline::Text {
                        value: " and use ] plus [ too".to_string(),
                    },
                ]),
                attrs: Some(Attributes {
                    id: None,
                    classes: Vec::new(),
                    pairs: expected_pairs,
                }),
            }]
        );
    }

    #[test]
    fn leaves_colon_sequences_without_a_letter_as_plain_text() {
        assert_eq!(
            parse_inlines("Note: this is 3:00pm and https://example.com stays linked."),
            vec![
                Inline::Text {
                    value: "Note: this is 3:00pm and ".to_string(),
                },
                Inline::AutoLink {
                    url: "https://example.com".to_string(),
                },
                Inline::Text {
                    value: " stays linked.".to_string(),
                },
            ]
        );
    }

    #[test]
    fn leaves_invalid_inline_directive_names_literal() {
        assert_eq!(
            parse_inlines(":widget_v2 :col3"),
            vec![Inline::Text {
                value: ":widget_v2 :col3".to_string(),
            }]
        );
    }

    #[test]
    fn treats_escaped_formatting_markers_as_literal_text() {
        assert_eq!(
            parse_inlines("\\*not italic\\* and \\~not sub\\~"),
            vec![Inline::Text {
                value: "*not italic* and ~not sub~".to_string(),
            }]
        );
    }
}
