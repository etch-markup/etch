use crate::{Block, ListItem, ParseError};
use std::iter::Peekable;

use super::ParseContext;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListMarker {
    Unordered,
    Ordered,
}

pub(crate) fn list_from_lines<'a, I>(
    first_line: &'a str,
    parent_indent: Option<usize>,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let marker = list_item_content(first_line)
        .map(|(_, marker, _, _)| marker)
        .unwrap_or(ListMarker::Unordered);
    let mut items = Vec::new();
    let mut next_first_line = Some(first_line);

    loop {
        let current_line = match next_first_line.take() {
            Some(line) => line,
            None => match lines.peek().copied() {
                Some((_, line)) if is_list_item_for_parent(line, parent_indent, marker) => {
                    lines.next();
                    line
                }
                _ => break,
            },
        };

        items.push(list_item_from_lines(
            current_line,
            parent_indent,
            marker,
            lines,
            errors,
            context.clone(),
        ));
    }

    Block::List {
        ordered: marker == ListMarker::Ordered,
        items,
        attrs: None,
    }
}

pub(crate) fn list_item_from_lines<'a, I>(
    first_line: &'a str,
    parent_indent: Option<usize>,
    marker: ListMarker,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> ListItem
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let Some((item_indent, _, checked, first_content)) = list_item_content(first_line) else {
        return ListItem {
            content: Vec::new(),
            checked: None,
        };
    };

    let mut continuation_lines = Vec::new();
    let mut pending_blank_lines = 0;

    while let Some((_, line)) = lines.peek().copied() {
        if line.trim().is_empty() {
            lines.next();
            pending_blank_lines += 1;
            continue;
        }

        if let Some((indent, next_marker, _, _)) = list_item_content(line) {
            if indent >= item_indent + 2 {
                lines.next();
                push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
                continuation_lines.push(strip_indent(line, item_indent + 2));
                continue;
            }

            if next_marker == marker && is_list_item_for_parent(line, parent_indent, marker) {
                break;
            }
        }

        if count_leading_spaces(line) >= item_indent + 2 {
            lines.next();
            push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
            continuation_lines.push(strip_indent(line, item_indent + 2));
            continue;
        }

        break;
    }

    let content = parse_list_item_blocks(first_content, &continuation_lines, errors, context);

    ListItem { content, checked }
}

pub(crate) fn parse_list_item_blocks<'a>(
    first_content: &'a str,
    continuation_lines: &[&'a str],
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Vec<Block> {
    let mut current = Vec::new();
    if !first_content.is_empty() {
        current.push(first_content);
    }

    let mut lines = continuation_lines.iter().copied().enumerate().peekable();
    super::parse_blocks_from_lines(
        &mut lines,
        false,
        true,
        &mut current,
        0,
        errors,
        None,
        context,
    )
    .0
}

pub(crate) fn push_item_blank_lines(lines: &mut Vec<&str>, pending_blank_lines: &mut usize) {
    for _ in 0..*pending_blank_lines {
        lines.push("");
    }

    *pending_blank_lines = 0;
}

pub(crate) fn list_parent_indent_for_block_start(
    line: &str,
    allow_indented_list_starts: bool,
) -> Option<Option<usize>> {
    match list_item_content(line) {
        Some((0, _, _, _)) => Some(None),
        Some(_) if allow_indented_list_starts => Some(Some(0)),
        _ => None,
    }
}

pub(crate) fn is_list_item_for_parent(
    line: &str,
    parent_indent: Option<usize>,
    marker: ListMarker,
) -> bool {
    let Some((indent, line_marker, _, _)) = list_item_content(line) else {
        return false;
    };

    if line_marker != marker {
        return false;
    }

    match parent_indent {
        Some(parent_indent) => indent >= parent_indent + 2,
        None => indent == 0,
    }
}

pub(crate) fn list_item_content(line: &str) -> Option<(usize, ListMarker, Option<bool>, &str)> {
    let indent = count_leading_spaces(line);
    let trimmed = &line[indent..];

    if let Some(content) = trimmed.strip_prefix("- ") {
        if let Some(content) = content.strip_prefix("[x] ") {
            return Some((indent, ListMarker::Unordered, Some(true), content));
        }

        if let Some(content) = content.strip_prefix("[ ] ") {
            return Some((indent, ListMarker::Unordered, Some(false), content));
        }

        return Some((indent, ListMarker::Unordered, None, content));
    }

    let digits = trimmed
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }

    trimmed[digits..]
        .strip_prefix(". ")
        .map(|content| (indent, ListMarker::Ordered, None, content))
}

pub(crate) fn count_leading_spaces(line: &str) -> usize {
    line.as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count()
}

pub(crate) fn strip_indent(line: &str, spaces: usize) -> &str {
    let actual = count_leading_spaces(line).min(spaces);
    &line[actual..]
}
