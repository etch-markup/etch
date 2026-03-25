use crate::Block;

use super::inline::parse_inlines;

pub(crate) fn paragraph_from_lines(lines: &[&str]) -> Block {
    Block::Paragraph {
        content: parse_inlines(&lines.join("\n")),
        attrs: None,
    }
}
