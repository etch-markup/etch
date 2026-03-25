use crate::Inline;

use super::{delimiter::count_delimiters, util::next_char_len};

pub(super) fn try_parse_inline_code(input: &str, index: usize) -> Option<(Inline, usize)> {
    let delimiter_len = count_delimiters(input, index, b'`');

    if !matches!(delimiter_len, 1 | 2) {
        return None;
    }

    let content_start = index + delimiter_len;
    let closing_index = find_closing_backticks(input, content_start, delimiter_len)?;

    Some((
        Inline::InlineCode {
            value: input[content_start..closing_index].to_string(),
        },
        closing_index + delimiter_len,
    ))
}

fn find_closing_backticks(input: &str, mut index: usize, delimiter_len: usize) -> Option<usize> {
    while index < input.len() {
        if input.as_bytes()[index] == b'`' && count_delimiters(input, index, b'`') >= delimiter_len
        {
            return Some(index);
        }

        index += next_char_len(input, index);
    }

    None
}
