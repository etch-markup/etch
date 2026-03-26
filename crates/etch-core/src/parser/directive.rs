use crate::{Attributes, Block, Inline, ParseError, ParseErrorKind};
use std::iter::Peekable;

use super::ParseContext;

pub(crate) struct DirectiveOpening {
    pub(crate) name: String,
    pub(crate) label: Option<Vec<Inline>>,
    pub(crate) attrs: Option<Attributes>,
    pub(crate) line: usize,
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
    if line.starts_with(":::") {
        return None;
    }

    directive_opening_from_line(line, "::", line_number)
}

pub(crate) fn container_directive_opening_from_line(
    line: &str,
    line_number: usize,
) -> Option<DirectiveOpening> {
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
    if remainder.starts_with('[') {
        let (label_text, next_remainder) = parse_balanced_bracket_segment(remainder)?;
        label = Some(vec![Inline::Text {
            value: label_text.to_string(),
        }]);
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
        name,
        label,
        attrs,
        line: line_number,
    })
}

pub(crate) fn block_directive_from_lines<'a, I>(
    opening: DirectiveOpening,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let mut body_lines = Vec::new();
    let mut nested_directives = Vec::new();
    let mut closed = false;

    while let Some((_, line)) = lines.next() {
        match nested_directives.last() {
            Some(NestedDirective::Block) if line == "::" => {
                nested_directives.pop();
                body_lines.push(line);
                continue;
            }
            Some(NestedDirective::Container)
                if line == ":::" || container_directive_named_close_from_line(line).is_some() =>
            {
                nested_directives.pop();
                body_lines.push(line);
                continue;
            }
            _ => {}
        }

        if line == "::" && nested_directives.is_empty() {
            closed = true;
            break;
        }

        if block_directive_opening_from_line(line, 0).is_some() {
            nested_directives.push(NestedDirective::Block);
        } else if container_directive_opening_from_line(line, 0).is_some() {
            nested_directives.push(NestedDirective::Container);
        }

        body_lines.push(line);
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
        name: opening.name,
        label: opening.label,
        attrs: opening.attrs,
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
        name: opening.name,
        label: opening.label,
        attrs: opening.attrs,
        body,
        named_close: matches!(close, Some(ContainerClose::Named)),
    }
}

pub(crate) fn container_directive_named_close_from_line(line: &str) -> Option<&str> {
    let remainder = line.strip_prefix(":::/")?;
    let name_len = directive_name_length(remainder)?;

    (name_len > 0 && name_len == remainder.len()).then_some(&remainder[..name_len])
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
