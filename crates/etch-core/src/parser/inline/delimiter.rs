use crate::{Inline, SourcePosition};

use super::{
    parse_segment,
    util::{char_after, char_before},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StarDelimiter {
    Emphasis,
    Strong,
    StrongEmphasis,
}

impl StarDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Emphasis => 1,
            Self::Strong => 2,
            Self::StrongEmphasis => 3,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Emphasis => Inline::Emphasis { content },
            Self::Strong => Inline::Strong { content },
            Self::StrongEmphasis => Inline::Strong {
                content: vec![Inline::Emphasis { content }],
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TildeDelimiter {
    Subscript,
    Strikethrough,
}

impl TildeDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Subscript => 1,
            Self::Strikethrough => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Subscript => Inline::Subscript { content },
            Self::Strikethrough => Inline::Strikethrough { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CaretDelimiter {
    Superscript,
}

impl CaretDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Superscript => 1,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Superscript => Inline::Superscript { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EqualDelimiter {
    Highlight,
}

impl EqualDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Highlight => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Highlight => Inline::Highlight { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PlusDelimiter {
    Insert,
}

impl PlusDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Insert => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Insert => Inline::Insert { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PipeDelimiter {
    Spoiler,
}

impl PipeDelimiter {
    fn len(self) -> usize {
        match self {
            Self::Spoiler => 2,
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Spoiler => Inline::Spoiler { content },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Delimiter {
    Star(StarDelimiter),
    Tilde(TildeDelimiter),
    Caret(CaretDelimiter),
    Equal(EqualDelimiter),
    Plus(PlusDelimiter),
    Pipe(PipeDelimiter),
}

impl Delimiter {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Star(delimiter) => delimiter.len(),
            Self::Tilde(delimiter) => delimiter.len(),
            Self::Caret(delimiter) => delimiter.len(),
            Self::Equal(delimiter) => delimiter.len(),
            Self::Plus(delimiter) => delimiter.len(),
            Self::Pipe(delimiter) => delimiter.len(),
        }
    }

    fn marker(self) -> u8 {
        match self {
            Self::Star(_) => b'*',
            Self::Tilde(_) => b'~',
            Self::Caret(_) => b'^',
            Self::Equal(_) => b'=',
            Self::Plus(_) => b'+',
            Self::Pipe(_) => b'|',
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Self::Star(delimiter) => delimiter.wrap(content),
            Self::Tilde(delimiter) => delimiter.wrap(content),
            Self::Caret(delimiter) => delimiter.wrap(content),
            Self::Equal(delimiter) => delimiter.wrap(content),
            Self::Plus(delimiter) => delimiter.wrap(content),
            Self::Pipe(delimiter) => delimiter.wrap(content),
        }
    }

    fn matches_run(self, run_len: usize) -> bool {
        match self {
            Self::Star(delimiter) => run_len >= delimiter.len(),
            Self::Tilde(delimiter) => run_len == delimiter.len(),
            Self::Caret(delimiter) => run_len == delimiter.len(),
            Self::Equal(delimiter) => run_len == delimiter.len(),
            Self::Plus(delimiter) => run_len == delimiter.len(),
            Self::Pipe(delimiter) => run_len == delimiter.len(),
        }
    }
}

pub(super) fn try_parse_delimiter_run(input: &str, index: usize) -> Option<(Inline, usize)> {
    let delimiter = parse_delimiter(input, index)?;

    if !can_open(input, index, delimiter) {
        return None;
    }

    let inner = parse_segment(
        input,
        index + delimiter.len(),
        SourcePosition { line: 1, column: 1 },
        Some(delimiter),
    );

    if inner.closed && !inner.nodes.is_empty() {
        return Some((delimiter.wrap(inner.nodes), inner.next_index));
    }

    None
}

fn parse_delimiter(input: &str, index: usize) -> Option<Delimiter> {
    let byte = input.as_bytes().get(index).copied()?;

    match byte {
        b'*' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Star(StarDelimiter::Emphasis)),
            2 => Some(Delimiter::Star(StarDelimiter::Strong)),
            3 => Some(Delimiter::Star(StarDelimiter::StrongEmphasis)),
            _ => None,
        },
        b'~' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Tilde(TildeDelimiter::Subscript)),
            2 => Some(Delimiter::Tilde(TildeDelimiter::Strikethrough)),
            _ => None,
        },
        b'^' => match count_delimiters(input, index, byte) {
            1 => Some(Delimiter::Caret(CaretDelimiter::Superscript)),
            _ => None,
        },
        b'=' => match count_delimiters(input, index, byte) {
            2 => Some(Delimiter::Equal(EqualDelimiter::Highlight)),
            _ => None,
        },
        b'+' => match count_delimiters(input, index, byte) {
            2 => Some(Delimiter::Plus(PlusDelimiter::Insert)),
            _ => None,
        },
        b'|' => match count_delimiters(input, index, byte) {
            2 => Some(Delimiter::Pipe(PipeDelimiter::Spoiler)),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn can_open(input: &str, index: usize, delimiter: Delimiter) -> bool {
    char_after(input, index + delimiter.len()).is_some_and(|ch| !ch.is_whitespace())
}

pub(super) fn can_close(
    input: &str,
    index: usize,
    delimiter: Delimiter,
    empty_content: bool,
) -> bool {
    !empty_content
        && delimiter.matches_run(count_delimiters(input, index, delimiter.marker()))
        && char_before(input, index).is_some_and(|ch| !ch.is_whitespace())
}

pub(super) fn count_delimiters(input: &str, index: usize, byte: u8) -> usize {
    input[index..]
        .bytes()
        .take_while(|candidate| *candidate == byte)
        .count()
}
