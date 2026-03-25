mod attributes;
mod blockquote;
mod code_block;
mod comment;
mod directive;
mod frontmatter;
mod heading;
mod inline;
mod list;
mod paragraph;
mod thematic_break;

use crate::{Block, Document, ParseError, ParseErrorKind, ParseResult};
use directive::ContainerClose;
use std::iter::Peekable;

use self::{
    blockquote::{blockquote_from_lines, is_blockquote_line},
    code_block::{code_block_from_lines, code_block_language_from_line},
    comment::strip_comments,
    directive::{
        block_directive_from_lines, block_directive_opening_from_line,
        container_directive_from_lines, container_directive_named_close_from_line,
        container_directive_opening_from_line,
    },
    frontmatter::parse_frontmatter,
    heading::heading_from_line,
    list::{list_from_lines, list_parent_indent_for_block_start},
    paragraph::paragraph_from_lines,
    thematic_break::thematic_break_from_line,
};

pub fn parse(input: &str) -> ParseResult {
    let (frontmatter, input_without_frontmatter) = parse_frontmatter(input);
    let body = strip_comments(input_without_frontmatter);
    let body_starts_at_document_start = frontmatter.is_none();
    let body_line_offset = input[..input.len() - input_without_frontmatter.len()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let mut errors = Vec::new();

    ParseResult {
        document: Document {
            frontmatter,
            body: parse_blocks(
                &body,
                body_starts_at_document_start,
                body_line_offset,
                &mut errors,
            ),
        },
        errors,
    }
}

pub(crate) fn parse_blocks(
    input: &str,
    body_starts_at_document_start: bool,
    line_offset: usize,
    errors: &mut Vec<ParseError>,
) -> Vec<Block> {
    let mut current = Vec::new();
    let mut lines = input.lines().enumerate().peekable();

    parse_blocks_from_lines(
        &mut lines,
        body_starts_at_document_start,
        false,
        &mut current,
        line_offset,
        errors,
        None,
    )
    .0
}

pub(crate) fn parse_blocks_from_lines<'a, I>(
    lines: &mut Peekable<I>,
    body_starts_at_document_start: bool,
    allow_indented_list_starts: bool,
    current: &mut Vec<&'a str>,
    line_offset: usize,
    errors: &mut Vec<ParseError>,
    container_name: Option<&str>,
) -> (Vec<Block>, Option<ContainerClose>)
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut blocks = Vec::new();

    while let Some((line_index, line)) = lines.next() {
        let line_number = line_offset + line_index + 1;

        if let Some(expected_container_name) = container_name {
            if line == ":::" {
                flush_paragraph(&mut blocks, current);
                return (blocks, Some(ContainerClose::Anonymous));
            }

            if let Some(close_name) = container_directive_named_close_from_line(line) {
                flush_paragraph(&mut blocks, current);

                if close_name != expected_container_name {
                    errors.push(ParseError {
                        kind: ParseErrorKind::Error,
                        message: format!(
                            "Mismatched container close: expected ':::/{}', found ':::/{}'",
                            expected_container_name, close_name
                        ),
                        line: line_number,
                        column: Some(1),
                    });
                }

                return (blocks, Some(ContainerClose::Named));
            }
        }

        if line.trim().is_empty() {
            flush_paragraph(&mut blocks, current);
            continue;
        }

        if let Some(language) = code_block_language_from_line(line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(code_block_from_lines(language, lines));
            continue;
        }

        if let Some(opening) = container_directive_opening_from_line(line, line_number) {
            flush_paragraph(&mut blocks, current);
            blocks.push(container_directive_from_lines(
                opening,
                lines,
                line_offset,
                errors,
            ));
            continue;
        }

        if let Some(opening) = block_directive_opening_from_line(line, line_number) {
            flush_paragraph(&mut blocks, current);
            blocks.push(block_directive_from_lines(opening, lines, errors));
            continue;
        }

        if let Some(heading) = heading_from_line(line) {
            flush_paragraph(&mut blocks, current);

            blocks.push(heading);
            continue;
        }

        if is_blockquote_line(line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(blockquote_from_lines(line, lines, errors));
            continue;
        }

        let is_first_document_line = body_starts_at_document_start && line_index == 0;
        if let Some(thematic_break) = thematic_break_from_line(line, is_first_document_line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(thematic_break);
            continue;
        }

        if let Some(parent_indent) =
            list_parent_indent_for_block_start(line, allow_indented_list_starts)
        {
            flush_paragraph(&mut blocks, current);
            blocks.push(list_from_lines(line, parent_indent, lines, errors));
            continue;
        }

        current.push(line);
    }

    flush_paragraph(&mut blocks, current);

    (blocks, None)
}

fn flush_paragraph<'a>(blocks: &mut Vec<Block>, current: &mut Vec<&'a str>) {
    if current.is_empty() {
        return;
    }

    blocks.push(paragraph_from_lines(current));
    current.clear();
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{Attributes, Block, Inline, ListItem, ParseError, ParseErrorKind};
    use serde_yaml::{Mapping, Value};
    use std::collections::HashMap;

    #[test]
    fn parse_wraps_input_in_a_single_paragraph_text_node() {
        let result = parse("hello");

        assert!(result.errors.is_empty());
        assert!(result.document.frontmatter.is_none());
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "hello".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_splits_paragraphs_on_blank_lines() {
        let result = parse("first paragraph\n\nsecond paragraph");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "first paragraph".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "second paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_extracts_frontmatter_from_the_very_first_line() {
        let result = parse("---\ntitle: \"Winter Notes\"\nauthor: trailwriter\n---\n\nBody");
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");

        assert_eq!(
            frontmatter.raw,
            "title: \"Winter Notes\"\nauthor: trailwriter\n"
        );
        assert_eq!(
            frontmatter.fields.get("title"),
            Some(&Value::String("Winter Notes".to_string()))
        );
        assert_eq!(
            frontmatter.fields.get("author"),
            Some(&Value::String("trailwriter".to_string()))
        );
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Body".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_empty_frontmatter() {
        let result = parse("---\n---\n\nBody");
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");

        assert_eq!(frontmatter.raw, "");
        assert!(frontmatter.fields.is_empty());
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Body".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_non_first_line_frontmatter_marker_as_a_thematic_break() {
        let result = parse("\n---\n\nBody");

        assert!(result.document.frontmatter.is_none());
        assert_eq!(
            result.document.body,
            vec![
                Block::ThematicBreak,
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_supports_nested_yaml_values_in_frontmatter() {
        let result = parse(
            "---\nseries:\n  name: \"Northern Passages\"\n  part: 3\ndraft: true\n---\n\nBody",
        );
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");
        let mut expected_series = Mapping::new();
        expected_series.insert(
            Value::String("name".to_string()),
            Value::String("Northern Passages".to_string()),
        );
        expected_series.insert(
            Value::String("part".to_string()),
            serde_yaml::to_value(3).expect("serializable integer"),
        );

        assert_eq!(
            frontmatter.fields.get("series"),
            Some(&Value::Mapping(expected_series))
        );
        assert_eq!(frontmatter.fields.get("draft"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_collapses_multiple_blank_lines_and_ignores_edge_whitespace() {
        let result = parse("\n \nfirst paragraph\n\n\t\n\nsecond paragraph\n  \n");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "first paragraph".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "second paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_emits_soft_breaks_for_multiline_paragraphs() {
        let result = parse("line one  \nline two\nline three");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![
                    Inline::Text {
                        value: "line one  ".to_string(),
                    },
                    Inline::SoftBreak,
                    Inline::Text {
                        value: "line two".to_string(),
                    },
                    Inline::SoftBreak,
                    Inline::Text {
                        value: "line three".to_string(),
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_headings_before_paragraph_fallback() {
        let result = parse("# Heading\nParagraph");

        assert_eq!(
            result.document.body,
            vec![
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text {
                        value: "Heading".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_thematic_breaks_before_paragraph_fallback() {
        let result = parse("before\n\n---\n\nafter");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "before".to_string(),
                    }],
                    attrs: None,
                },
                Block::ThematicBreak,
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "after".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_fenced_code_blocks_before_paragraph_fallback() {
        let result = parse("before\n\n```rust\nfn main() {}\n```\n\nafter");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "before".to_string(),
                    }],
                    attrs: None,
                },
                Block::CodeBlock {
                    language: Some("rust".to_string()),
                    content: "fn main() {}".to_string(),
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "after".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_blockquotes_before_paragraph_fallback() {
        let result = parse("> quoted\n\noutside");

        assert_eq!(
            result.document.body,
            vec![
                Block::BlockQuote {
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "quoted".to_string(),
                        }],
                        attrs: None,
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "outside".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_splits_blockquote_paragraphs_on_quoted_blank_lines() {
        let result = parse("> first paragraph\n>\n> second paragraph");

        assert_eq!(
            result.document.body,
            vec![Block::BlockQuote {
                content: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "first paragraph".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "second paragraph".to_string(),
                        }],
                        attrs: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_nested_blockquotes_by_recursively_parsing_stripped_content() {
        let result = parse("> outer\n>> inner\n>>> deepest");

        assert_eq!(
            result.document.body,
            vec![Block::BlockQuote {
                content: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "outer".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::BlockQuote {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "inner".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::BlockQuote {
                                content: vec![Block::Paragraph {
                                    content: vec![Inline::Text {
                                        value: "deepest".to_string(),
                                    }],
                                    attrs: None,
                                }],
                                attrs: None,
                            },
                        ],
                        attrs: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_keeps_fenced_code_block_content_raw() {
        let result = parse("```\n**bold**\n:note[keep literal]\n{~ not a comment ~}\n```");

        assert_eq!(
            result.document.body,
            vec![Block::CodeBlock {
                language: None,
                content: "**bold**\n:note[keep literal]\n{~ not a comment ~}".to_string(),
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_strips_inline_comments_from_paragraph_text() {
        let result = parse("Text before {~ this is hidden ~} text after.");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Text before  text after.".to_string(),
                }],
                attrs: None,
            }]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_strips_full_line_comments_between_blocks() {
        let result = parse("Paragraph before.\n\n{~ hidden line ~}\n\nParagraph after.");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph before.".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph after.".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_strips_multi_line_comments_between_blocks() {
        let result = parse(
            "Paragraph before.\n\n{~\nHidden line one.\nHidden line two.\n~}\n\nParagraph after.",
        );

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph before.".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph after.".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_treats_comment_openers_inside_comments_as_literal_text() {
        let result = parse(
            "Before {~ outer comment {~ this looks like inner ~} but the comment closed at the first ~} and this is visible text ~}.",
        );

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Before  but the comment closed at the first ~} and this is visible text ~}."
                        .to_string(),
                }],
                attrs: None,
            }]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_detects_basic_block_directives_before_paragraph_fallback() {
        let result = parse("::aside\nInside the aside.\n::");

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: None,
                attrs: None,
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Inside the aside.".to_string(),
                    }],
                    attrs: None,
                }],
            }]
        );
    }

    #[test]
    fn parse_treats_blank_lines_inside_block_directives_as_body_content() {
        let result = parse("::aside\nFirst paragraph.\n\nSecond paragraph.\n::");

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: None,
                attrs: None,
                body: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "First paragraph.".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Second paragraph.".to_string(),
                        }],
                        attrs: None,
                    },
                ],
            }]
        );
    }

    #[test]
    fn parse_supports_block_directive_labels_and_attributes() {
        let result = parse("::aside[Author note]{#callout .highlight tone=\"quiet\"}\nBody\n::");
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("tone".to_string(), "quiet".to_string());

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: Some(vec![Inline::Text {
                    value: "Author note".to_string(),
                }]),
                attrs: Some(Attributes {
                    id: Some("callout".to_string()),
                    classes: vec!["highlight".to_string()],
                    pairs: expected_pairs,
                }),
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                }],
            }]
        );
    }

    #[test]
    fn parse_rejects_block_directive_names_with_non_letter_non_hyphen_characters() {
        let result = parse("::note2\nBody\n::");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![
                    Inline::Text {
                        value: "::note2".to_string(),
                    },
                    Inline::SoftBreak,
                    Inline::Text {
                        value: "Body".to_string(),
                    },
                    Inline::SoftBreak,
                    Inline::Text {
                        value: "::".to_string(),
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_container_directives_with_anonymous_closes() {
        let result = parse(":::chapter\nInside the chapter.\n:::");

        assert_eq!(
            result.document.body,
            vec![Block::ContainerDirective {
                name: "chapter".to_string(),
                label: None,
                attrs: None,
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Inside the chapter.".to_string(),
                    }],
                    attrs: None,
                }],
                named_close: false,
            }]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_supports_container_directives_with_named_closes_and_attrs() {
        let result =
            parse(":::chapter[One]{#intro .wide title=\"Lantern Watch\"}\nBody\n:::/chapter");
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("title".to_string(), "Lantern Watch".to_string());

        assert_eq!(
            result.document.body,
            vec![Block::ContainerDirective {
                name: "chapter".to_string(),
                label: Some(vec![Inline::Text {
                    value: "One".to_string(),
                }]),
                attrs: Some(Attributes {
                    id: Some("intro".to_string()),
                    classes: vec!["wide".to_string()],
                    pairs: expected_pairs,
                }),
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                }],
                named_close: true,
            }]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_supports_nested_container_directives() {
        let result = parse(":::chapter\n:::section\nNested body.\n:::/section\n:::/chapter");

        assert_eq!(
            result.document.body,
            vec![Block::ContainerDirective {
                name: "chapter".to_string(),
                label: None,
                attrs: None,
                body: vec![Block::ContainerDirective {
                    name: "section".to_string(),
                    label: None,
                    attrs: None,
                    body: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Nested body.".to_string(),
                        }],
                        attrs: None,
                    }],
                    named_close: true,
                }],
                named_close: true,
            }]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_reports_mismatched_container_named_closes() {
        let result = parse(":::chapter\nBody\n:::/section");

        assert_eq!(
            result.document.body,
            vec![Block::ContainerDirective {
                name: "chapter".to_string(),
                label: None,
                attrs: None,
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                }],
                named_close: true,
            }]
        );
        assert_eq!(
            result.errors,
            vec![ParseError {
                kind: ParseErrorKind::Error,
                message: "Mismatched container close: expected ':::/chapter', found ':::/section'"
                    .to_string(),
                line: 3,
                column: Some(1),
            }]
        );
    }

    #[test]
    fn parse_supports_empty_fenced_code_blocks() {
        let result = parse("```\n```");

        assert_eq!(
            result.document.body,
            vec![Block::CodeBlock {
                language: None,
                content: String::new(),
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_accepts_all_thematic_break_markers_with_optional_spaces() {
        let result = parse("- - -\n***\n_ _ _ _");

        assert_eq!(
            result.document.body,
            vec![
                Block::ThematicBreak,
                Block::ThematicBreak,
                Block::ThematicBreak,
            ]
        );
    }

    #[test]
    fn parse_preserves_first_line_frontmatter_marker_as_non_thematic_break() {
        let result = parse("---");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "---".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_invalid_thematic_break_lines_as_paragraph_text() {
        let result = parse("--\n-*-");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![
                    Inline::Text {
                        value: "--".to_string(),
                    },
                    Inline::SoftBreak,
                    Inline::Text {
                        value: "-*-".to_string(),
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_invalid_hash_prefixes_as_paragraph_text() {
        let result = parse("#no-space\n\n####### Too many hashes");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "#no-space".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "####### Too many hashes".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_ignores_the_required_leading_test_description_comment() {
        let result = parse("{~ test description ~}\n\nparagraph");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "paragraph".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_returns_an_empty_body_for_whitespace_only_content() {
        let result = parse(" \n\t\n  ");

        assert!(result.document.body.is_empty());
    }

    #[test]
    fn parse_detects_unordered_lists_before_paragraph_fallback() {
        let result = parse("- pack rope\n- fill canteen");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "pack rope".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "fill canteen".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_checked_and_unchecked_task_list_items() {
        let result = parse("- [x] Reserve the campsite\n- [ ] Print the route map");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Reserve the campsite".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: Some(true),
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Print the route map".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: Some(false),
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_indented_lines_as_unordered_list_item_continuations() {
        let result = parse(
            "- Camp briefing for the new arrivals.\n  Bring dry socks, a flashlight, and a map.\n\n  Check in before sunset.",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![
                        Block::Paragraph {
                            content: vec![
                                Inline::Text {
                                    value: "Camp briefing for the new arrivals.".to_string(),
                                },
                                Inline::SoftBreak,
                                Inline::Text {
                                    value: "Bring dry socks, a flashlight, and a map.".to_string(),
                                }
                            ],
                            attrs: None,
                        },
                        Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Check in before sunset.".to_string(),
                            }],
                            attrs: None,
                        },
                    ],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_parses_nested_blocks_inside_list_item_content() {
        let result = parse(
            "- Camp briefing for the new arrivals.\n  Bring dry socks and a flashlight.\n\n  > Check in before sunset.\n  > Keep your permit visible.",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![
                        Block::Paragraph {
                            content: vec![
                                Inline::Text {
                                    value: "Camp briefing for the new arrivals.".to_string(),
                                },
                                Inline::SoftBreak,
                                Inline::Text {
                                    value: "Bring dry socks and a flashlight.".to_string(),
                                },
                            ],
                            attrs: None,
                        },
                        Block::BlockQuote {
                            content: vec![Block::Paragraph {
                                content: vec![
                                    Inline::Text {
                                        value: "Check in before sunset.".to_string(),
                                    },
                                    Inline::SoftBreak,
                                    Inline::Text {
                                        value: "Keep your permit visible.".to_string(),
                                    },
                                ],
                                attrs: None,
                            }],
                            attrs: None,
                        },
                    ],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_keeps_first_list_item_line_as_paragraph_text_when_it_starts_with_block_syntax() {
        let result = parse("- # not a heading");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "# not a heading".to_string(),
                        }],
                        attrs: None,
                    }],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_nested_unordered_lists_with_lenient_indentation() {
        let result = parse(
            "- Weekend project\n  - Buy lumber\n    - Measure each board twice\n    - Mark the cut lines clearly\n- Workshop cleanup\n    - Sweep the sawdust",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Weekend project".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::List {
                                ordered: false,
                                items: vec![ListItem {
                                    content: vec![
                                        Block::Paragraph {
                                            content: vec![Inline::Text {
                                                value: "Buy lumber".to_string(),
                                            }],
                                            attrs: None,
                                        },
                                        Block::List {
                                            ordered: false,
                                            items: vec![
                                                ListItem {
                                                    content: vec![Block::Paragraph {
                                                        content: vec![Inline::Text {
                                                            value: "Measure each board twice"
                                                                .to_string(),
                                                        }],
                                                        attrs: None,
                                                    }],
                                                    checked: None,
                                                },
                                                ListItem {
                                                    content: vec![Block::Paragraph {
                                                        content: vec![Inline::Text {
                                                            value: "Mark the cut lines clearly"
                                                                .to_string(),
                                                        }],
                                                        attrs: None,
                                                    }],
                                                    checked: None,
                                                },
                                            ],
                                            attrs: None,
                                        },
                                    ],
                                    checked: None,
                                }],
                                attrs: None,
                            },
                        ],
                        checked: None,
                    },
                    ListItem {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Workshop cleanup".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::List {
                                ordered: false,
                                items: vec![ListItem {
                                    content: vec![Block::Paragraph {
                                        content: vec![Inline::Text {
                                            value: "Sweep the sawdust".to_string(),
                                        }],
                                        attrs: None,
                                    }],
                                    checked: None,
                                }],
                                attrs: None,
                            },
                        ],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_ordered_lists_before_paragraph_fallback() {
        let result = parse("1. Preheat the oven\n2. Chop the carrots");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: true,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Preheat the oven".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Chop the carrots".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_separates_ordered_lists_paragraphs_and_unordered_lists() {
        let result = parse(
            "1. Boil the water\n2. Add the tea leaves\n\nThe kettle can rest here.\n\n- Fold the camp blanket\n- Lock the storage bin",
        );

        assert_eq!(
            result.document.body,
            vec![
                Block::List {
                    ordered: true,
                    items: vec![
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Boil the water".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Add the tea leaves".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                    ],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "The kettle can rest here.".to_string(),
                    }],
                    attrs: None,
                },
                Block::List {
                    ordered: false,
                    items: vec![
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Fold the camp blanket".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Lock the storage bin".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                    ],
                    attrs: None,
                },
            ]
        );
    }
}
