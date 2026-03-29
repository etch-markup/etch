mod code;
mod delimiter;
mod directive;
mod escape;
mod link;
mod util;

use crate::{Inline, SourcePosition};

use self::{
    code::try_parse_inline_code,
    delimiter::{Delimiter, can_close, count_delimiters, try_parse_delimiter_run},
    directive::try_parse_inline_directive,
    escape::{try_parse_escaped_literal, try_parse_hard_break, try_parse_soft_break},
    link::{try_parse_autolink, try_parse_footnote_reference, try_parse_image, try_parse_link},
    util::{next_char_len, push_text},
};

struct ParseResult {
    nodes: Vec<Inline>,
    next_index: usize,
    closed: bool,
}

#[allow(dead_code)]
pub(crate) fn parse_inlines(input: &str) -> Vec<Inline> {
    parse_inlines_with_position(input, 1, 1)
}

pub(crate) fn parse_inlines_with_position(
    input: &str,
    start_line: usize,
    start_column: usize,
) -> Vec<Inline> {
    parse_segment(
        input,
        0,
        SourcePosition {
            line: start_line,
            column: start_column,
        },
        None,
    )
    .nodes
}

pub(super) fn advance_position(mut position: SourcePosition, text: &str) -> SourcePosition {
    for ch in text.chars() {
        if ch == '\n' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }
    }

    position
}

fn parse_segment(
    input: &str,
    mut index: usize,
    mut position: SourcePosition,
    stop: Option<Delimiter>,
) -> ParseResult {
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
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            if let Some(next_index) = try_parse_escaped_literal(input, index) {
                push_text(&mut nodes, &input[text_start..index]);
                push_text(&mut nodes, &input[index + 1..next_index]);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }
        }

        if let Some(next_index) = try_parse_soft_break(input, index) {
            push_text(&mut nodes, &input[text_start..index]);
            nodes.push(Inline::SoftBreak);
            position = advance_position(position, &input[index..next_index]);
            index = next_index;
            text_start = index;
            continue;
        }

        if byte == b'h' {
            if let Some((inline, next_index)) = try_parse_autolink(input, index) {
                push_text(&mut nodes, &input[text_start..index]);
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }
        }

        if byte == b'!' && input.as_bytes().get(index + 1).copied() == Some(b'[') {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_image(input, index) {
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            position = advance_position(position, &input[index..index + 1]);
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

            if let Some((inline, next_index)) = try_parse_link(input, index, position) {
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            position = advance_position(position, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        if byte == b':' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_inline_directive(input, index, position) {
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            position = advance_position(position, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        if byte == b'`' {
            push_text(&mut nodes, &input[text_start..index]);
            let run_len = count_delimiters(input, index, b'`');

            if let Some((inline, next_index)) = try_parse_inline_code(input, index) {
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + run_len]);
            position = advance_position(position, &input[index..index + run_len]);
            index += run_len;
            text_start = index;
            continue;
        }

        if byte == b'*' || byte == b'~' || byte == b'^' || byte == b'=' || byte == b'+' || byte == b'|' {
            push_text(&mut nodes, &input[text_start..index]);

            if let Some((inline, next_index)) = try_parse_delimiter_run(input, index) {
                nodes.push(inline);
                position = advance_position(position, &input[index..next_index]);
                index = next_index;
                text_start = index;
                continue;
            }

            push_text(&mut nodes, &input[index..index + 1]);
            position = advance_position(position, &input[index..index + 1]);
            index += 1;
            text_start = index;
            continue;
        }

        let next_index = index + next_char_len(input, index);
        position = advance_position(position, &input[index..next_index]);
        index = next_index;
    }

    push_text(&mut nodes, &input[text_start..index]);

    ParseResult {
        nodes,
        next_index: index,
        closed: false,
    }
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
    fn parses_adjacent_mixed_inline_formats_with_spaces() {
        assert_eq!(
            parse_inlines("*a* **b** `c` ~~d~~"),
            vec![
                Inline::Emphasis {
                    content: vec![Inline::Text {
                        value: "a".to_string(),
                    }],
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::Strong {
                    content: vec![Inline::Text {
                        value: "b".to_string(),
                    }],
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::InlineCode {
                    value: "c".to_string(),
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::Strikethrough {
                    content: vec![Inline::Text {
                        value: "d".to_string(),
                    }],
                },
            ]
        );
    }

    #[test]
    fn parses_adjacent_mixed_inline_formats_without_spaces() {
        assert_eq!(
            parse_inlines("*a***b**`c`~~d~~"),
            vec![
                Inline::Emphasis {
                    content: vec![Inline::Text {
                        value: "a".to_string(),
                    }],
                },
                Inline::Strong {
                    content: vec![Inline::Text {
                        value: "b".to_string(),
                    }],
                },
                Inline::InlineCode {
                    value: "c".to_string(),
                },
                Inline::Strikethrough {
                    content: vec![Inline::Text {
                        value: "d".to_string(),
                    }],
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
    fn parses_cross_line_formatting_with_soft_breaks() {
        assert_eq!(
            parse_inlines("==high\nlight== ++in\nsert++ ^super\nscript^ ~sub\nscript~"),
            vec![
                Inline::Highlight {
                    content: vec![
                        Inline::Text {
                            value: "high".to_string(),
                        },
                        Inline::SoftBreak,
                        Inline::Text {
                            value: "light".to_string(),
                        },
                    ],
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::Insert {
                    content: vec![
                        Inline::Text {
                            value: "in".to_string(),
                        },
                        Inline::SoftBreak,
                        Inline::Text {
                            value: "sert".to_string(),
                        },
                    ],
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::Superscript {
                    content: vec![
                        Inline::Text {
                            value: "super".to_string(),
                        },
                        Inline::SoftBreak,
                        Inline::Text {
                            value: "script".to_string(),
                        },
                    ],
                },
                Inline::Text {
                    value: " ".to_string(),
                },
                Inline::Subscript {
                    content: vec![
                        Inline::Text {
                            value: "sub".to_string(),
                        },
                        Inline::SoftBreak,
                        Inline::Text {
                            value: "script".to_string(),
                        },
                    ],
                },
            ]
        );
    }

    #[test]
    fn parses_cross_line_inline_code_spans() {
        assert_eq!(
            parse_inlines("before `line one\nline two` after"),
            vec![
                Inline::Text {
                    value: "before ".to_string(),
                },
                Inline::InlineCode {
                    value: "line one\nline two".to_string(),
                },
                Inline::Text {
                    value: " after".to_string(),
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

        let nodes = parse_inlines("I met :character[Sable]{species=fox mood=playful} nearby.");
        assert_eq!(
            nodes[0],
            Inline::Text {
                value: "I met ".to_string()
            }
        );
        assert_eq!(
            nodes[2],
            Inline::Text {
                value: " nearby.".to_string()
            }
        );
        let Inline::InlineDirective {
            directive_id,
            name,
            content,
            raw_content,
            attrs,
            ..
        } = &nodes[1]
        else {
            panic!("expected inline directive");
        };
        assert_eq!(*directive_id, 1);
        assert_eq!(name, "character");
        assert_eq!(raw_content.as_deref(), Some("Sable"));
        assert_eq!(
            content,
            &Some(vec![Inline::Text {
                value: "Sable".to_string(),
            }])
        );
        assert_eq!(
            attrs,
            &Some(Attributes {
                id: None,
                classes: Vec::new(),
                pairs: expected_pairs,
            })
        );
    }

    #[test]
    fn parses_bare_inline_directives() {
        let nodes = parse_inlines("Insert :pagebreak here.\n:toc");
        assert_eq!(
            nodes[0],
            Inline::Text {
                value: "Insert ".to_string()
            }
        );
        assert_eq!(
            nodes[2],
            Inline::Text {
                value: " here.".to_string()
            }
        );
        assert_eq!(nodes[3], Inline::SoftBreak);
        let Inline::InlineDirective {
            directive_id,
            name,
            content,
            raw_content,
            attrs,
            ..
        } = &nodes[1]
        else {
            panic!("expected pagebreak directive");
        };
        assert_eq!(
            (*directive_id, name.as_str(), content, raw_content, attrs),
            (1, "pagebreak", &None, &None, &None)
        );
        let Inline::InlineDirective {
            directive_id,
            name,
            content,
            raw_content,
            attrs,
            ..
        } = &nodes[4]
        else {
            panic!("expected toc directive");
        };
        assert_eq!(
            (*directive_id, name.as_str(), content, raw_content, attrs),
            (2, "toc", &None, &None, &None)
        );
    }

    #[test]
    fn parses_inline_directive_content_with_formatting_and_escaped_brackets() {
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("text".to_string(), "extra info".to_string());

        let nodes =
            parse_inlines(":tooltip[**bold** and use \\] plus \\[ too]{text=\"extra info\"}");
        let Inline::InlineDirective {
            directive_id,
            name,
            content,
            raw_content,
            attrs,
            ..
        } = &nodes[0]
        else {
            panic!("expected tooltip directive");
        };
        assert_eq!(*directive_id, 1);
        assert_eq!(name, "tooltip");
        assert_eq!(
            raw_content.as_deref(),
            Some("**bold** and use \\] plus \\[ too")
        );
        assert_eq!(
            content,
            &Some(vec![
                Inline::Strong {
                    content: vec![Inline::Text {
                        value: "bold".to_string(),
                    }],
                },
                Inline::Text {
                    value: " and use ] plus [ too".to_string(),
                },
            ])
        );
        assert_eq!(
            attrs,
            &Some(Attributes {
                id: None,
                classes: Vec::new(),
                pairs: expected_pairs,
            })
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
            parse_inlines(
                "\\*not italic\\* \\~not sub\\~ \\^not super\\^ \\=\\=not highlight\\=\\= \\+\\+not insert\\+\\+ \\[brackets\\] \\\\"
            ),
            vec![Inline::Text {
                value: "*not italic* ~not sub~ ^not super^ ==not highlight== ++not insert++ [brackets] \\".to_string(),
            }]
        );
    }

    #[test]
    fn treats_escaped_markers_as_literal_inside_formatted_content() {
        assert_eq!(
            parse_inlines("*literal \\* star and \\[brackets\\]*"),
            vec![Inline::Emphasis {
                content: vec![Inline::Text {
                    value: "literal * star and [brackets]".to_string(),
                }],
            }]
        );
    }

    #[test]
    fn does_not_process_escapes_inside_code_spans() {
        assert_eq!(
            parse_inlines("`\\*not italic\\* and \\[literal\\] \\\\`"),
            vec![Inline::InlineCode {
                value: "\\*not italic\\* and \\[literal\\] \\\\".to_string(),
            }]
        );
    }
}
