use crate::Block;

pub(crate) fn thematic_break_from_line(line: &str, is_first_document_line: bool) -> Option<Block> {
    if is_first_document_line && line == "---" {
        return None;
    }

    let trimmed = line.trim_matches(' ');
    let mut characters = trimmed.chars();
    let marker = characters.next()?;

    if !matches!(marker, '-' | '*' | '_') {
        return None;
    }

    let mut marker_count = 1;

    for ch in characters {
        match ch {
            ' ' => {}
            current if current == marker => marker_count += 1,
            _ => return None,
        }
    }

    (marker_count >= 3).then_some(Block::ThematicBreak)
}
