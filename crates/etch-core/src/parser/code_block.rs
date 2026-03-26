use crate::Block;
use std::iter::Peekable;

pub(crate) fn code_block_language_from_line(line: &str) -> Option<Option<String>> {
    if !line.starts_with("```") {
        return None;
    }

    let language = line[3..].trim();
    Some((!language.is_empty()).then_some(language.to_string()))
}

pub(crate) fn code_block_from_lines<'a, I>(
    language: Option<String>,
    lines: &mut Peekable<I>,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut content = Vec::new();

    for (_, line) in &mut *lines {
        if line == "```" {
            break;
        }

        content.push(line);
    }

    let attrs = super::attributes::take_attribute_only_line(lines);

    Block::CodeBlock {
        language,
        content: content.join("\n"),
        attrs,
    }
}
