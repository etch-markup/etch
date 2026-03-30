use crate::{Attributes, Block, Inline, ParseError, ParseErrorKind, SourcePosition, SourceSpan};
use std::iter::Peekable;

use super::{ParseContext, next_directive_id};

pub(crate) struct DirectiveOpening {
    pub(crate) directive_id: u64,
    pub(crate) name: String,
    pub(crate) label: Option<Vec<Inline>>,
    pub(crate) raw_label: Option<String>,
    pub(crate) attrs: Option<Attributes>,
    pub(crate) line: usize,
    pub(crate) span_start: SourcePosition,
}

pub(crate) enum ContainerClose {
    Anonymous,
    Named,
}

enum NestedDirective {
    Block,
    Container,
}

pub(crate) fn block_directive_opening_from_line(
    line: &str,
    line_number: usize,
) -> Option<DirectiveOpening> {
    if !is_block_directive_opening_line(line) {
        return None;
    }

    directive_opening_from_line(line, "::", line_number)
}

pub(crate) fn container_directive_opening_from_line(
    line: &str,
    line_number: usize,
) -> Option<DirectiveOpening> {
    if !is_container_directive_opening_line(line) {
        return None;
    }

    directive_opening_from_line(line, ":::", line_number)
}

pub(crate) fn directive_opening_from_line(
    line: &str,
    prefix: &str,
    line_number: usize,
) -> Option<DirectiveOpening> {
    let mut remainder = line.strip_prefix(prefix)?;
    if !remainder
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }

    let name_len = directive_name_length(remainder)?;
    if name_len == 0 {
        return None;
    }

    let name = remainder[..name_len].to_string();
    remainder = &remainder[name_len..];

    let mut label = None;
    let mut raw_label = None;
    if remainder.starts_with('[') {
        let (label_text, next_remainder) = parse_balanced_bracket_segment(remainder)?;
        label = Some(vec![Inline::Text {
            value: label_text.to_string(),
        }]);
        raw_label = Some(label_text.to_string());
        remainder = next_remainder;
    }

    let mut attrs = None;
    if remainder.starts_with('{') {
        let (parsed_attrs, next_remainder) =
            super::attributes::parse_attributes_segment(remainder)?;
        attrs = Some(parsed_attrs);
        remainder = next_remainder;
    }

    remainder.is_empty().then_some(DirectiveOpening {
        directive_id: next_directive_id(),
        name,
        label,
        raw_label,
        attrs,
        line: line_number,
        span_start: SourcePosition {
            line: line_number,
            column: 1,
        },
    })
}

pub(crate) fn block_directive_from_lines<'a, I>(
    opening: DirectiveOpening,
    lines: &mut Peekable<I>,
    line_offset: usize,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let mut body_lines = Vec::new();
    let mut nested_directives = Vec::new();
    let mut closed = false;
    let mut end = SourcePosition {
        line: opening.line,
        column: 1,
    };

    for (line_index, line) in lines.by_ref() {
        let line_number = line_offset + line_index + 1;
        match nested_directives.last() {
            Some(NestedDirective::Block) if line == "::" => {
                nested_directives.pop();
                body_lines.push(line);
                end = SourcePosition {
                    line: line_number,
                    column: line.chars().count() + 1,
                };
                continue;
            }
            Some(NestedDirective::Container)
                if line == ":::" || container_directive_named_close_from_line(line).is_some() =>
            {
                nested_directives.pop();
                body_lines.push(line);
                end = SourcePosition {
                    line: line_number,
                    column: line.chars().count() + 1,
                };
                continue;
            }
            _ => {}
        }

        if line == "::" && nested_directives.is_empty() {
            closed = true;
            end = SourcePosition {
                line: line_number,
                column: line.chars().count() + 1,
            };
            break;
        }

        if is_block_directive_opening_line(line) {
            nested_directives.push(NestedDirective::Block);
        } else if is_container_directive_opening_line(line) {
            nested_directives.push(NestedDirective::Container);
        }

        body_lines.push(line);
        end = SourcePosition {
            line: line_number,
            column: line.chars().count() + 1,
        };
    }

    if !closed {
        errors.push(ParseError {
            kind: ParseErrorKind::Error,
            message: format!(
                "unclosed ::{} started on line {}",
                opening.name, opening.line
            ),
            line: opening.line,
            column: Some(1),
        });
    }

    Block::BlockDirective {
        directive_id: opening.directive_id,
        span: SourceSpan {
            start: opening.span_start,
            end,
        },
        name: opening.name,
        label: opening.label,
        raw_label: opening.raw_label,
        attrs: opening.attrs,
        raw_body: body_lines.join("\n"),
        body: super::parse_blocks(&body_lines.join("\n"), false, opening.line, errors, context),
    }
}

pub(crate) fn container_directive_from_lines<'a, I>(
    opening: DirectiveOpening,
    lines: &mut Peekable<I>,
    line_offset: usize,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let (raw_body, end, named_close) =
        collect_container_body_metadata(lines.clone(), line_offset, &opening.name);
    let mut current = Vec::new();
    let (body, close) = super::parse_blocks_from_lines(
        lines,
        false,
        false,
        &mut current,
        line_offset,
        errors,
        Some(&opening.name),
        context,
    );

    if close.is_none() {
        errors.push(ParseError {
            kind: ParseErrorKind::Error,
            message: format!(
                "unclosed :::{} started on line {}",
                opening.name, opening.line
            ),
            line: opening.line,
            column: Some(1),
        });
    }

    Block::ContainerDirective {
        directive_id: opening.directive_id,
        span: SourceSpan {
            start: opening.span_start,
            end,
        },
        name: opening.name,
        label: opening.label,
        raw_label: opening.raw_label,
        attrs: opening.attrs,
        raw_body,
        body,
        named_close: named_close || matches!(close, Some(ContainerClose::Named)),
    }
}

fn collect_container_body_metadata<'a, I>(
    lines: Peekable<I>,
    line_offset: usize,
    container_name: &str,
) -> (String, SourcePosition, bool)
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let mut raw_lines = Vec::new();
    let mut nested_containers = 0usize;
    let mut end = SourcePosition {
        line: line_offset + 1,
        column: 1,
    };
    let mut named_close = false;

    for (line_index, line) in lines {
        let line_number = line_offset + line_index + 1;

        if line == ":::" {
            if nested_containers == 0 {
                end = SourcePosition {
                    line: line_number,
                    column: line.chars().count() + 1,
                };
                break;
            }

            nested_containers -= 1;
            raw_lines.push(line);
            continue;
        }

        if let Some(close_name) = container_directive_named_close_from_line(line) {
            if nested_containers == 0 {
                named_close = close_name == container_name;
                end = SourcePosition {
                    line: line_number,
                    column: line.chars().count() + 1,
                };
                break;
            }

            if nested_containers > 0 {
                nested_containers -= 1;
                raw_lines.push(line);
                continue;
            }
        }

        if is_container_directive_opening_line(line) {
            nested_containers += 1;
        }

        raw_lines.push(line);
        end = SourcePosition {
            line: line_number,
            column: line.chars().count() + 1,
        };
    }

    (raw_lines.join("\n"), end, named_close)
}

pub(crate) fn container_directive_named_close_from_line(line: &str) -> Option<&str> {
    let remainder = line.strip_prefix(":::/")?;
    let name_len = directive_name_length(remainder)?;

    (name_len > 0 && name_len == remainder.len()).then_some(&remainder[..name_len])
}

fn is_block_directive_opening_line(line: &str) -> bool {
    if line.starts_with(":::") {
        return false;
    }

    let Some(remainder) = line.strip_prefix("::") else {
        return false;
    };
    remainder
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
        && directive_name_length(remainder).is_some_and(|length| length > 0)
}

fn is_container_directive_opening_line(line: &str) -> bool {
    let Some(remainder) = line.strip_prefix(":::") else {
        return false;
    };
    remainder
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
        && directive_name_length(remainder).is_some_and(|length| length > 0)
}

pub(crate) fn directive_name_length(input: &str) -> Option<usize> {
    let mut length = 0;

    for (index, ch) in input.char_indices() {
        if ch.is_ascii_alphabetic() || ch == '-' {
            length = index + ch.len_utf8();
            continue;
        }

        return matches!(ch, '[' | '{').then_some(length);
    }

    Some(length)
}

pub(crate) fn parse_balanced_bracket_segment(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            if ch != '[' {
                return None;
            }

            depth = 1;
            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&input[1..index], &input[index + 1..]));
                }
            }
            _ => {}
        }
    }

    None
}
