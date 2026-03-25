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

struct ParseResult {
    nodes: Vec<Inline>,
    next_index: usize,
    closed: bool,
}

#[allow(dead_code)]
pub(crate) fn parse_inlines(input: &str) -> Vec<Inline> {
    parse_segment(input, 0, None).nodes
}

fn parse_segment(input: &str, mut index: usize, stop: Option<StarDelimiter>) -> ParseResult {
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

        if input.as_bytes()[index] == b'*' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_star_run(input, index) {
                nodes.push(inline);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, "*");
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

fn try_parse_star_run(input: &str, index: usize) -> Option<(Inline, usize)> {
    let run_len = count_stars(input, index);
    let delimiter = match run_len {
        1 => StarDelimiter::Emphasis,
        2 => StarDelimiter::Strong,
        3 => StarDelimiter::StrongEmphasis,
        _ => return None,
    };

    if !can_open(input, index, delimiter) {
        return None;
    }

    let inner = parse_segment(input, index + delimiter.len(), Some(delimiter));

    if inner.closed && !inner.nodes.is_empty() {
        return Some((delimiter.wrap(inner.nodes), inner.next_index));
    }

    None
}

fn can_open(input: &str, index: usize, delimiter: StarDelimiter) -> bool {
    char_after(input, index + delimiter.len()).is_some_and(|ch| !ch.is_whitespace())
}

fn can_close(input: &str, index: usize, delimiter: StarDelimiter, empty_content: bool) -> bool {
    !empty_content
        && count_stars(input, index) >= delimiter.len()
        && char_before(input, index).is_some_and(|ch| !ch.is_whitespace())
}

fn count_stars(input: &str, index: usize) -> usize {
    input[index..]
        .bytes()
        .take_while(|byte| *byte == b'*')
        .count()
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
    fn leaves_empty_or_whitespace_delimiters_literal() {
        assert_eq!(
            parse_inlines("** **** * text* *text *"),
            vec![Inline::Text {
                value: "** **** * text* *text *".to_string(),
            }]
        );
    }
}
