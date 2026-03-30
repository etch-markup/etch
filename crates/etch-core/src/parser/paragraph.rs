use crate::Block;

use super::{attributes::split_trailing_block_attributes, inline::parse_inlines};

pub(crate) fn paragraph_from_lines(lines: &[&str]) -> Block {
    let mut content_lines = lines.to_vec();
    let mut attrs = None;

    if let Some(last_line) = content_lines.last_mut()
        && let Some((content_without_attrs, parsed_attrs)) =
            split_trailing_block_attributes(last_line)
    {
        *last_line = content_without_attrs;
        attrs = Some(parsed_attrs);
    }

    Block::Paragraph {
        content: parse_inlines(&content_lines.join("\n")),
        attrs,
    }
}
