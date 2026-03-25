use super::util::char_after;

pub(super) fn try_parse_hard_break(input: &str, index: usize) -> Option<usize> {
    let remainder = input.get(index..)?;

    if remainder.starts_with("\\\r\n") {
        return Some(index + "\\\r\n".len());
    }

    remainder
        .starts_with("\\\n")
        .then_some(index + "\\\n".len())
}

pub(super) fn try_parse_escaped_literal(input: &str, index: usize) -> Option<usize> {
    let escaped = char_after(input, index + 1)?;

    matches!(escaped, '*' | '~' | '^' | '=' | '+' | '[' | ']' | '\\')
        .then_some(index + 1 + escaped.len_utf8())
}

pub(super) fn try_parse_soft_break(input: &str, index: usize) -> Option<usize> {
    let remainder = input.get(index..)?;

    if remainder.starts_with("\r\n") {
        return Some(index + "\r\n".len());
    }

    if remainder.starts_with('\n') || remainder.starts_with('\r') {
        return Some(index + 1);
    }

    None
}
