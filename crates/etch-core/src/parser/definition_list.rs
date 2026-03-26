use crate::{Block, DefinitionItem, ParseError};
use std::iter::Peekable;

use super::{
    ParseContext,
    inline::parse_inlines,
    list::{count_leading_spaces, parse_list_item_blocks, push_item_blank_lines, strip_indent},
};

pub(crate) fn definition_opening_from_line(line: &str) -> Option<&str> {
    line.strip_prefix(": ")
}

pub(crate) fn definition_list_starts_with<'a, I>(term_line: &str, lines: &mut Peekable<I>) -> bool
where
    I: Iterator<Item = (usize, &'a str)>,
{
    if term_line.trim().is_empty() || definition_opening_from_line(term_line).is_some() {
        return false;
    }

    let Some((_, next_line)) = lines.peek().copied() else {
        return false;
    };

    definition_opening_from_line(next_line).is_some()
}

pub(crate) fn definition_list_from_lines<'a, I>(
    first_term: &'a str,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Option<Block>
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    if !definition_list_starts_with(first_term, lines) {
        return None;
    }

    let mut items = Vec::new();
    let mut next_term = Some(first_term);

    loop {
        let term = next_term.take().expect("term should be queued");
        let mut definitions = Vec::new();

        while let Some((_, line)) = lines.peek().copied() {
            let Some(first_content) = definition_opening_from_line(line) else {
                break;
            };

            lines.next();
            definitions.push(definition_blocks_from_lines(
                first_content,
                lines,
                errors,
                context.clone(),
            ));
        }

        items.push(DefinitionItem {
            term: parse_inlines(term),
            definitions,
        });

        let Some((blank_lines, term_line)) = next_definition_term(lines) else {
            break;
        };

        for _ in 0..blank_lines {
            lines.next();
        }
        lines.next();
        next_term = Some(term_line);
    }

    Some(Block::DefinitionList { items, attrs: None })
}

fn definition_blocks_from_lines<'a, I>(
    first_content: &'a str,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Vec<Block>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut continuation_lines = Vec::new();
    let mut pending_blank_lines = 0;

    while let Some((_, line)) = lines.peek().copied() {
        if line.trim().is_empty() {
            lines.next();
            pending_blank_lines += 1;
            continue;
        }

        if count_leading_spaces(line) < 2 {
            break;
        }

        lines.next();
        push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
        continuation_lines.push(strip_indent(line, 2));
    }

    parse_list_item_blocks(
        first_content.trim_start(),
        &continuation_lines,
        errors,
        context,
    )
}

fn next_definition_term<'a, I>(lines: &Peekable<I>) -> Option<(usize, &'a str)>
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let mut lookahead = lines.clone();
    let mut blank_lines = 0;

    while let Some((_, line)) = lookahead.next() {
        if line.trim().is_empty() {
            blank_lines += 1;
            continue;
        }

        let Some((_, next_line)) = lookahead.peek().copied() else {
            return None;
        };

        return definition_opening_from_line(next_line).map(|_| (blank_lines, line));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        definition_list_from_lines, definition_list_starts_with, definition_opening_from_line,
    };
    use crate::parser::ParseContext;
    use crate::{Block, DefinitionItem, Inline};

    #[test]
    fn detects_definition_opening_lines() {
        assert_eq!(
            definition_opening_from_line(": A portable light."),
            Some("A portable light.")
        );
        assert_eq!(definition_opening_from_line(":not a definition"), None);
    }

    #[test]
    fn detects_definition_list_starts_from_term_plus_definition_lookahead() {
        let mut lines = [(0usize, ": Definition"), (1usize, "Next paragraph")]
            .into_iter()
            .peekable();

        assert!(definition_list_starts_with("Lantern", &mut lines));
        assert!(!definition_list_starts_with(": Definition", &mut lines));
    }

    #[test]
    fn parses_multiple_terms_and_multiple_definitions() {
        let mut lines = [
            (0usize, ": A current of cold air."),
            (1usize, ": An unfinished version of a document."),
            (2usize, ""),
            (3usize, "Lantern"),
            (4usize, ": A portable light."),
            (5usize, "After"),
        ]
        .into_iter()
        .peekable();
        let mut errors = Vec::new();

        assert_eq!(
            definition_list_from_lines("Draft", &mut lines, &mut errors, ParseContext::root()),
            Some(Block::DefinitionList {
                items: vec![
                    DefinitionItem {
                        term: vec![Inline::Text {
                            value: "Draft".to_string(),
                        }],
                        definitions: vec![
                            vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "A current of cold air.".to_string(),
                                }],
                                attrs: None,
                            }],
                            vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "An unfinished version of a document.".to_string(),
                                }],
                                attrs: None,
                            }],
                        ],
                    },
                    DefinitionItem {
                        term: vec![Inline::Text {
                            value: "Lantern".to_string(),
                        }],
                        definitions: vec![vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "A portable light.".to_string(),
                            }],
                            attrs: None,
                        }]],
                    },
                ],
                attrs: None,
            })
        );

        assert_eq!(lines.next(), Some((5usize, "After")));
        assert!(errors.is_empty());
    }

    #[test]
    fn parses_indented_definition_continuations_as_blocks() {
        let mut lines = [
            (0usize, ": First paragraph"),
            (1usize, "  still the first paragraph."),
            (2usize, ""),
            (3usize, "  - nested item"),
            (4usize, "Outside"),
        ]
        .into_iter()
        .peekable();
        let mut errors = Vec::new();

        assert_eq!(
            definition_list_from_lines("Term", &mut lines, &mut errors, ParseContext::root()),
            Some(Block::DefinitionList {
                items: vec![DefinitionItem {
                    term: vec![Inline::Text {
                        value: "Term".to_string(),
                    }],
                    definitions: vec![vec![
                        Block::Paragraph {
                            content: vec![
                                Inline::Text {
                                    value: "First paragraph".to_string(),
                                },
                                Inline::SoftBreak,
                                Inline::Text {
                                    value: "still the first paragraph.".to_string(),
                                },
                            ],
                            attrs: None,
                        },
                        Block::List {
                            ordered: false,
                            items: vec![crate::ListItem {
                                content: vec![Block::Paragraph {
                                    content: vec![Inline::Text {
                                        value: "nested item".to_string(),
                                    }],
                                    attrs: None,
                                }],
                                checked: None,
                            }],
                            attrs: None,
                        },
                    ]],
                }],
                attrs: None,
            })
        );

        assert_eq!(lines.next(), Some((4usize, "Outside")));
        assert!(errors.is_empty());
    }
}
