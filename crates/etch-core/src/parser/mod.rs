mod attributes;
mod directive;
mod frontmatter;
mod inline;

use crate::{Block, Document, Inline, ParseResult};

pub fn parse(input: &str) -> ParseResult {
    ParseResult {
        document: Document {
            frontmatter: None,
            body: vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: input.to_string(),
                }],
                attrs: None,
            }],
        },
        errors: Vec::new(),
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
}
