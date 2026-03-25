use crate::Inline;

pub(super) fn next_char_len(input: &str, index: usize) -> usize {
    input[index..]
        .chars()
        .next()
        .expect("index points to a valid character boundary")
        .len_utf8()
}

pub(super) fn char_after(input: &str, index: usize) -> Option<char> {
    input.get(index..)?.chars().next()
}

pub(super) fn char_before(input: &str, index: usize) -> Option<char> {
    input.get(..index)?.chars().next_back()
}

pub(super) fn push_text(nodes: &mut Vec<Inline>, value: &str) {
    if value.is_empty() {
        return;
    }

    if let Some(Inline::Text { value: existing }) = nodes.last_mut() {
        existing.push_str(value);
        return;
    }

    nodes.push(Inline::Text {
        value: value.to_string(),
    });
}
