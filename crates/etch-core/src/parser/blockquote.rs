use crate::{Block, ParseError};
use std::iter::Peekable;

use super::ParseContext;

pub(crate) fn blockquote_from_lines<'a, I>(
    first_line: &'a str,
    lines: &mut Peekable<I>,
    errors: &mut Vec<ParseError>,
    context: ParseContext,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut content = vec![strip_blockquote_marker(first_line)];

    while let Some((_, line)) = lines.next_if(|(_, line)| is_blockquote_line(line)) {
        content.push(strip_blockquote_marker(line));
    }

    Block::BlockQuote {
        content: super::parse_blocks(&content.join("\n"), false, 0, errors, context),
        attrs: None,
    }
}

pub(crate) fn is_blockquote_line(line: &str) -> bool {
    line.starts_with('>')
}

pub(crate) fn strip_blockquote_marker(line: &str) -> &str {
    let Some(remainder) = line.strip_prefix('>') else {
        return line;
    };

    remainder.strip_prefix(' ').unwrap_or(remainder)
}
