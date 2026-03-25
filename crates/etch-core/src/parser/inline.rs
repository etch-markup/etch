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

        if byte == b'[' {
            push_text(&mut nodes, &input[text_start..index]);

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
    use crate::Inline;

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
    fn does_not_parse_images_as_links() {
        assert_eq!(
            parse_inlines("![alt](https://docs.etch-lang.dev/image.png)"),
            vec![Inline::Text {
                value: "![alt](https://docs.etch-lang.dev/image.png)".to_string(),
            }]
        );
    }

    #[test]
    fn does_not_parse_footnote_references_as_links() {
        assert_eq!(
            parse_inlines("See [^guide] later"),
            vec![Inline::Text {
                value: "See [^guide] later".to_string(),
            }]
        );
    }
}
