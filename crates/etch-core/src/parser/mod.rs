mod attributes;
mod directive;
mod frontmatter;
mod inline;

use crate::{Block, Document, Inline, ParseResult};

pub fn parse(input: &str) -> ParseResult {
    ParseResult {
        document: Document {
            frontmatter: None,
            body: parse_paragraphs(skip_leading_comment(input)),
        },
        errors: Vec::new(),
    }
}

fn parse_paragraphs(input: &str) -> Vec<Block> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in input.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(paragraph_from_lines(&current));
                current.clear();
            }
            continue;
        }

        current.push(line);
    }

    if !current.is_empty() {
        paragraphs.push(paragraph_from_lines(&current));
    }

    paragraphs
}

fn paragraph_from_lines(lines: &[&str]) -> Block {
    Block::Paragraph {
        content: vec![Inline::Text {
            value: lines.join("\n"),
        }],
        attrs: None,
    }
}

fn skip_leading_comment(input: &str) -> &str {
    if !input.starts_with("{~") {
        return input;
    }

    let Some(comment_end) = input.find("~}") else {
        return input;
    };

    let mut remainder = &input[comment_end + 2..];

    while let Some(next_line_end) = remainder.find('\n') {
        let line = &remainder[..next_line_end];
        if !line.trim().is_empty() {
            break;
        }

        remainder = &remainder[next_line_end + 1..];
    }

    if remainder.trim().is_empty() {
        ""
    } else {
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{Block, Inline};

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
    fn parse_keeps_multiline_paragraph_text_together() {
        let result = parse("line one  \nline two\nline three");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "line one  \nline two\nline three".to_string(),
                }],
                attrs: None,
            }]
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
}
