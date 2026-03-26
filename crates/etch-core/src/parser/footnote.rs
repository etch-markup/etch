use crate::{Block, ParseError};
use std::iter::Peekable;

use super::ParseContext;
use super::list::{
    count_leading_spaces, parse_list_item_blocks, push_item_blank_lines, strip_indent,
};

pub(crate) fn footnote_definition_opening_from_line(line: &str) -> Option<(String, &str)> {
    let trimmed = line.strip_prefix("[^")?;
    let label_end = trimmed.find("]:")?;
    let label = &trimmed[..label_end];

    if label.is_empty() {
        return None;
    }

    Some((label.to_string(), &trimmed[label_end + 2..]))
}

pub(crate) fn footnote_definition_from_lines<'a, I>(
    first_line: &'a str,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Option<Block>
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let (label, first_content) = footnote_definition_opening_from_line(first_line)?;
    let mut continuation_lines = Vec::new();
    let mut pending_blank_lines = 0;

    while let Some((_, line)) = lines.peek().copied() {
        if line.trim().is_empty() {
            lines.next();
            pending_blank_lines += 1;
            continue;
        }

        if count_leading_spaces(line) < 2 {
            if pending_blank_lines == 0 && is_lazy_footnote_continuation(line) {
                lines.next();
                continuation_lines.push(line);
                continue;
            }

            break;
        }

        lines.next();
        push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
        continuation_lines.push(strip_indent(line, 2));
    }

    Some(Block::FootnoteDefinition {
        label,
        content: parse_list_item_blocks(
            first_content.trim_start(),
            &continuation_lines,
            errors,
            context,
        ),
    })
}

fn is_lazy_footnote_continuation(line: &str) -> bool {
    if line == "::"
        || line == ":::"
        || super::directive::container_directive_named_close_from_line(line).is_some()
    {
        return false;
    }

    super::code_block::code_block_language_from_line(line).is_none()
        && super::directive::container_directive_opening_from_line(line, 0).is_none()
        && super::directive::block_directive_opening_from_line(line, 0).is_none()
        && super::heading::heading_from_line(line).is_none()
        && footnote_definition_opening_from_line(line).is_none()
        && !super::blockquote::is_blockquote_line(line)
        && super::thematic_break::thematic_break_from_line(line, false).is_none()
        && super::list::list_parent_indent_for_block_start(line, false).is_none()
        && super::definition_list::definition_opening_from_line(line).is_none()
        && !line.trim_start().starts_with('|')
}

#[cfg(test)]
mod tests {
    use super::{footnote_definition_from_lines, footnote_definition_opening_from_line};
    use crate::parser::ParseContext;
    use crate::{Block, Inline};

    #[test]
    fn parses_footnote_definition_opening() {
        assert_eq!(
            footnote_definition_opening_from_line("[^note]: Details here."),
            Some(("note".to_string(), " Details here."))
        );
    }

    #[test]
    fn rejects_non_footnote_lines() {
        assert_eq!(
            footnote_definition_opening_from_line("[note]: Details here."),
            None
        );
        assert_eq!(
            footnote_definition_opening_from_line("[^]: Details here."),
            None
        );
    }

    #[test]
    fn parses_multi_paragraph_footnote_definition_blocks() {
        let mut lines = [
            (1usize, "  still the first paragraph."),
            (2usize, ""),
            (3usize, "  Second paragraph."),
            (4usize, "[^next]: not part of this footnote"),
        ]
        .into_iter()
        .peekable();
        let mut errors = Vec::new();

        assert_eq!(
            footnote_definition_from_lines(
                "[^note]: Opening line",
                &mut lines,
                &mut errors,
                ParseContext::root(),
            ),
            Some(Block::FootnoteDefinition {
                label: "note".to_string(),
                content: vec![
                    Block::Paragraph {
                        content: vec![
                            Inline::Text {
                                value: "Opening line".to_string(),
                            },
                            Inline::SoftBreak,
                            Inline::Text {
                                value: "still the first paragraph.".to_string(),
                            },
                        ],
                        attrs: None,
                    },
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Second paragraph.".to_string(),
                        }],
                        attrs: None,
                    },
                ],
            })
        );

        assert_eq!(
            lines.next(),
            Some((4usize, "[^next]: not part of this footnote"))
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn parses_lazy_footnote_continuation_lines() {
        let mut lines = [(1usize, "folktale about returning wolves.")]
            .into_iter()
            .peekable();
        let mut errors = Vec::new();

        assert_eq!(
            footnote_definition_from_lines(
                "[^1]: The finding is loosely based on a Scandinavian",
                &mut lines,
                &mut errors,
                ParseContext::root(),
            ),
            Some(Block::FootnoteDefinition {
                label: "1".to_string(),
                content: vec![Block::Paragraph {
                    content: vec![
                        Inline::Text {
                            value: "The finding is loosely based on a Scandinavian".to_string(),
                        },
                        Inline::SoftBreak,
                        Inline::Text {
                            value: "folktale about returning wolves.".to_string(),
                        },
                    ],
                    attrs: None,
                }],
            })
        );

        assert!(lines.next().is_none());
        assert!(errors.is_empty());
    }
}
