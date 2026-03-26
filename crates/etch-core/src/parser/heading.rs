use crate::Block;

use super::{attributes::split_trailing_block_attributes, inline::parse_inlines};

pub(crate) fn heading_from_line(line: &str) -> Option<Block> {
    let hash_count = line.chars().take_while(|ch| *ch == '#').count();

    if !(1..=6).contains(&hash_count) {
        return None;
    }

    if line.chars().nth(hash_count) != Some(' ') {
        return None;
    }

    let content = &line[hash_count + 1..];
    let (content, attrs) = match split_trailing_block_attributes(content) {
        Some((content_without_attrs, attrs)) => (content_without_attrs, Some(attrs)),
        None => (content, None),
    };

    Some(Block::Heading {
        level: hash_count as u8,
        content: parse_inlines(content),
        attrs,
    })
}
