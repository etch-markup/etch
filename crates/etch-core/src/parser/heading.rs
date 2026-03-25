use crate::{Block, Inline};

pub(crate) fn heading_from_line(line: &str) -> Option<Block> {
    let hash_count = line.chars().take_while(|ch| *ch == '#').count();

    if !(1..=6).contains(&hash_count) {
        return None;
    }

    if line.chars().nth(hash_count) != Some(' ') {
        return None;
    }

    Some(Block::Heading {
        level: hash_count as u8,
        content: vec![Inline::Text {
            value: line[hash_count + 1..].to_string(),
        }],
        attrs: None,
    })
}
