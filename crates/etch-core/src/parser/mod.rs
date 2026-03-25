use crate::{Attributes, Block, Document, Frontmatter, Inline, ListItem, ParseResult};
use std::{collections::HashMap, iter::Peekable};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListMarker {
    Unordered,
    Ordered,
}

struct BlockDirectiveOpening {
    name: String,
    label: Option<Vec<Inline>>,
    attrs: Option<Attributes>,
}

pub fn parse(input: &str) -> ParseResult {
    let (frontmatter, input_without_frontmatter) = parse_frontmatter(input);
    let body = skip_leading_comment(input_without_frontmatter);
    let body_starts_at_document_start =
        frontmatter.is_none() && std::ptr::eq(body.as_ptr(), input.as_ptr());

    ParseResult {
        document: Document {
            frontmatter,
            body: parse_blocks(body, body_starts_at_document_start),
        },
        errors: Vec::new(),
    }
}

fn parse_frontmatter(input: &str) -> (Option<Frontmatter>, &str) {
    let Some((first_line, rest, opener_len)) = split_first_line(input) else {
        return (None, input);
    };

    if first_line != "---" {
        return (None, input);
    }

    let mut remaining = rest;
    let mut line_start = opener_len;

    while let Some((line, next_rest, line_len)) = split_first_line(remaining) {
        if line == "---" {
            let raw = input[opener_len..line_start].to_string();
            let fields = if raw.is_empty() {
                HashMap::new()
            } else {
                serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(&raw).unwrap_or_default()
            };

            return (Some(Frontmatter { raw, fields }), next_rest);
        }

        line_start += line_len;
        remaining = next_rest;
    }

    (None, input)
}

fn split_first_line(input: &str) -> Option<(&str, &str, usize)> {
    if input.is_empty() {
        return None;
    }

    if let Some(newline_index) = input.find('\n') {
        let line = input[..newline_index]
            .strip_suffix('\r')
            .unwrap_or(&input[..newline_index]);
        return Some((line, &input[newline_index + 1..], newline_index + 1));
    }

    Some((input.strip_suffix('\r').unwrap_or(input), "", input.len()))
}

fn parse_blocks(input: &str, body_starts_at_document_start: bool) -> Vec<Block> {
    let mut current = Vec::new();
    let mut lines = input.lines().enumerate().peekable();

    parse_blocks_from_lines(
        &mut lines,
        body_starts_at_document_start,
        false,
        &mut current,
    )
}

fn parse_blocks_from_lines<'a, I>(
    lines: &mut Peekable<I>,
    body_starts_at_document_start: bool,
    allow_indented_list_starts: bool,
    current: &mut Vec<&'a str>,
) -> Vec<Block>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut blocks = Vec::new();

    while let Some((line_index, line)) = lines.next() {
        if line.trim().is_empty() {
            flush_paragraph(&mut blocks, current);
            continue;
        }

        if let Some(language) = code_block_language_from_line(line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(code_block_from_lines(language, lines));
            continue;
        }

        if let Some(opening) = block_directive_opening_from_line(line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(block_directive_from_lines(opening, lines));
            continue;
        }

        if let Some(heading) = heading_from_line(line) {
            flush_paragraph(&mut blocks, current);

            blocks.push(heading);
            continue;
        }

        if is_blockquote_line(line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(blockquote_from_lines(line, lines));
            continue;
        }

        let is_first_document_line = body_starts_at_document_start && line_index == 0;
        if let Some(thematic_break) = thematic_break_from_line(line, is_first_document_line) {
            flush_paragraph(&mut blocks, current);
            blocks.push(thematic_break);
            continue;
        }

        if let Some(parent_indent) =
            list_parent_indent_for_block_start(line, allow_indented_list_starts)
        {
            flush_paragraph(&mut blocks, current);
            blocks.push(list_from_lines(line, parent_indent, lines));
            continue;
        }

        current.push(line);
    }

    flush_paragraph(&mut blocks, current);

    blocks
}

fn code_block_language_from_line(line: &str) -> Option<Option<String>> {
    if !line.starts_with("```") {
        return None;
    }

    let language = line[3..].trim();
    Some((!language.is_empty()).then_some(language.to_string()))
}

fn code_block_from_lines<'a, I>(language: Option<String>, lines: &mut I) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut content = Vec::new();

    for (_, line) in lines {
        if line == "```" {
            break;
        }

        content.push(line);
    }

    Block::CodeBlock {
        language,
        content: content.join("\n"),
        attrs: None,
    }
}

fn block_directive_opening_from_line(line: &str) -> Option<BlockDirectiveOpening> {
    if line.starts_with(":::") {
        return None;
    }

    let mut remainder = line.strip_prefix("::")?;
    if !remainder
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }

    let name_len = directive_name_length(remainder)?;
    if name_len == 0 {
        return None;
    }

    let name = remainder[..name_len].to_string();
    remainder = &remainder[name_len..];

    let mut label = None;
    if remainder.starts_with('[') {
        let (label_text, next_remainder) = parse_balanced_bracket_segment(remainder)?;
        label = Some(vec![Inline::Text {
            value: label_text.to_string(),
        }]);
        remainder = next_remainder;
    }

    let mut attrs = None;
    if remainder.starts_with('{') {
        let (parsed_attrs, next_remainder) = parse_attributes_segment(remainder)?;
        attrs = Some(parsed_attrs);
        remainder = next_remainder;
    }

    remainder
        .is_empty()
        .then_some(BlockDirectiveOpening { name, label, attrs })
}

fn block_directive_from_lines<'a, I>(
    opening: BlockDirectiveOpening,
    lines: &mut Peekable<I>,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut body_lines = Vec::new();

    while let Some((_, line)) = lines.next() {
        if line == "::" {
            break;
        }

        body_lines.push(line);
    }

    Block::BlockDirective {
        name: opening.name,
        label: opening.label,
        attrs: opening.attrs,
        body: parse_blocks(&body_lines.join("\n"), false),
    }
}

fn directive_name_length(input: &str) -> Option<usize> {
    let mut length = 0;

    for (index, ch) in input.char_indices() {
        if ch.is_ascii_alphabetic() || ch == '-' {
            length = index + ch.len_utf8();
            continue;
        }

        return matches!(ch, '[' | '{').then_some(length);
    }

    Some(length)
}

fn parse_balanced_bracket_segment(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            if ch != '[' {
                return None;
            }

            depth = 1;
            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&input[1..index], &input[index + 1..]));
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_attributes_segment(input: &str) -> Option<(Attributes, &str)> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            if ch != '{' {
                return None;
            }

            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => in_quotes = !in_quotes,
            '}' if !in_quotes => {
                let attrs = parse_attributes_content(&input[1..index])?;
                return Some((attrs, &input[index + 1..]));
            }
            _ => {}
        }
    }

    None
}

fn parse_attributes_content(input: &str) -> Option<Attributes> {
    let mut attrs = Attributes {
        id: None,
        classes: Vec::new(),
        pairs: HashMap::new(),
    };

    for token in split_attribute_tokens(input) {
        if token.is_empty() {
            continue;
        }

        if let Some(id) = token.strip_prefix('#') {
            if id.is_empty() {
                return None;
            }

            attrs.id = Some(id.to_string());
            continue;
        }

        if let Some(class) = token.strip_prefix('.') {
            if class.is_empty() {
                return None;
            }

            attrs.classes.push(class.to_string());
            continue;
        }

        let (key, value) = token.split_once('=')?;
        if key.is_empty() {
            return None;
        }

        let value = if value.starts_with('"') {
            let quoted = value.strip_prefix('"')?.strip_suffix('"')?;
            unescape_quoted_attribute_value(quoted)
        } else {
            value.to_string()
        };

        attrs.pairs.insert(key.to_string(), value);
    }

    Some(attrs)
}

fn split_attribute_tokens(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => {
                in_quotes = !in_quotes;
                start.get_or_insert(index);
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if let Some(token_start) = start.take() {
                    tokens.push(&input[token_start..index]);
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if let Some(token_start) = start {
        tokens.push(&input[token_start..]);
    }

    tokens
}

fn unescape_quoted_attribute_value(value: &str) -> String {
    let mut unescaped = String::with_capacity(value.len());
    let mut escaped = false;

    for ch in value.chars() {
        if escaped {
            unescaped.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        unescaped.push(ch);
    }

    if escaped {
        unescaped.push('\\');
    }

    unescaped
}

fn blockquote_from_lines<'a, I>(first_line: &'a str, lines: &mut Peekable<I>) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut content = vec![strip_blockquote_marker(first_line)];

    while let Some((_, line)) = lines.next_if(|(_, line)| is_blockquote_line(line)) {
        content.push(strip_blockquote_marker(line));
    }

    Block::BlockQuote {
        content: parse_blocks(&content.join("\n"), false),
        attrs: None,
    }
}

fn is_blockquote_line(line: &str) -> bool {
    line.starts_with('>')
}

fn strip_blockquote_marker(line: &str) -> &str {
    let Some(remainder) = line.strip_prefix('>') else {
        return line;
    };

    remainder.strip_prefix(' ').unwrap_or(remainder)
}

fn heading_from_line(line: &str) -> Option<Block> {
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

fn paragraph_from_lines(lines: &[&str]) -> Block {
    Block::Paragraph {
        content: vec![Inline::Text {
            value: lines.join("\n"),
        }],
        attrs: None,
    }
}

fn list_from_lines<'a, I>(
    first_line: &'a str,
    parent_indent: Option<usize>,
    lines: &mut Peekable<I>,
) -> Block
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let marker = list_item_content(first_line)
        .map(|(_, marker, _, _)| marker)
        .unwrap_or(ListMarker::Unordered);
    let mut items = Vec::new();
    let mut next_first_line = Some(first_line);

    loop {
        let current_line = match next_first_line.take() {
            Some(line) => line,
            None => match lines.peek().copied() {
                Some((_, line)) if is_list_item_for_parent(line, parent_indent, marker) => {
                    lines.next();
                    line
                }
                _ => break,
            },
        };

        items.push(list_item_from_lines(
            current_line,
            parent_indent,
            marker,
            lines,
        ));
    }

    Block::List {
        ordered: marker == ListMarker::Ordered,
        items,
        attrs: None,
    }
}

fn list_item_from_lines<'a, I>(
    first_line: &'a str,
    parent_indent: Option<usize>,
    marker: ListMarker,
    lines: &mut Peekable<I>,
) -> ListItem
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let Some((item_indent, _, checked, first_content)) = list_item_content(first_line) else {
        return ListItem {
            content: Vec::new(),
            checked: None,
        };
    };

    let mut continuation_lines = Vec::new();
    let mut pending_blank_lines = 0;

    while let Some((_, line)) = lines.peek().copied() {
        if line.trim().is_empty() {
            lines.next();
            pending_blank_lines += 1;
            continue;
        }

        if let Some((indent, next_marker, _, _)) = list_item_content(line) {
            if indent >= item_indent + 2 {
                lines.next();
                push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
                continuation_lines.push(strip_indent(line, item_indent + 2));
                continue;
            }

            if next_marker == marker && is_list_item_for_parent(line, parent_indent, marker) {
                break;
            }
        }

        if count_leading_spaces(line) >= item_indent + 2 {
            lines.next();
            push_item_blank_lines(&mut continuation_lines, &mut pending_blank_lines);
            continuation_lines.push(strip_indent(line, item_indent + 2));
            continue;
        }

        break;
    }

    let content = parse_list_item_blocks(first_content, &continuation_lines);

    ListItem { content, checked }
}

fn parse_list_item_blocks<'a>(
    first_content: &'a str,
    continuation_lines: &[&'a str],
) -> Vec<Block> {
    let mut current = Vec::new();
    if !first_content.is_empty() {
        current.push(first_content);
    }

    let mut lines = continuation_lines.iter().copied().enumerate().peekable();
    parse_blocks_from_lines(&mut lines, false, true, &mut current)
}

fn push_item_blank_lines(lines: &mut Vec<&str>, pending_blank_lines: &mut usize) {
    for _ in 0..*pending_blank_lines {
        lines.push("");
    }

    *pending_blank_lines = 0;
}

fn list_parent_indent_for_block_start(
    line: &str,
    allow_indented_list_starts: bool,
) -> Option<Option<usize>> {
    match list_item_content(line) {
        Some((0, _, _, _)) => Some(None),
        Some(_) if allow_indented_list_starts => Some(Some(0)),
        _ => None,
    }
}

fn is_list_item_for_parent(line: &str, parent_indent: Option<usize>, marker: ListMarker) -> bool {
    let Some((indent, line_marker, _, _)) = list_item_content(line) else {
        return false;
    };

    if line_marker != marker {
        return false;
    }

    match parent_indent {
        Some(parent_indent) => indent >= parent_indent + 2,
        None => indent == 0,
    }
}

fn list_item_content(line: &str) -> Option<(usize, ListMarker, Option<bool>, &str)> {
    let indent = count_leading_spaces(line);
    let trimmed = &line[indent..];

    if let Some(content) = trimmed.strip_prefix("- ") {
        if let Some(content) = content.strip_prefix("[x] ") {
            return Some((indent, ListMarker::Unordered, Some(true), content));
        }

        if let Some(content) = content.strip_prefix("[ ] ") {
            return Some((indent, ListMarker::Unordered, Some(false), content));
        }

        return Some((indent, ListMarker::Unordered, None, content));
    }

    let digits = trimmed
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }

    trimmed[digits..]
        .strip_prefix(". ")
        .map(|content| (indent, ListMarker::Ordered, None, content))
}

fn count_leading_spaces(line: &str) -> usize {
    line.as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count()
}

fn strip_indent(line: &str, spaces: usize) -> &str {
    let actual = count_leading_spaces(line).min(spaces);
    &line[actual..]
}

fn thematic_break_from_line(line: &str, is_first_document_line: bool) -> Option<Block> {
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

fn flush_paragraph<'a>(blocks: &mut Vec<Block>, current: &mut Vec<&'a str>) {
    if current.is_empty() {
        return;
    }

    blocks.push(paragraph_from_lines(current));
    current.clear();
}

fn skip_leading_comment(input: &str) -> &str {
    if !input.starts_with("{~") {
        return input;
    }

    let Some(comment_end) = input.find("~}") else {
        return input;
    };

    let mut remainder = &input[comment_end + 2..];

    while let Some(next_line_end) = remainder.find('\n') {
        let line = &remainder[..next_line_end];
        if !line.trim().is_empty() {
            break;
        }

        remainder = &remainder[next_line_end + 1..];
    }

    if remainder.trim().is_empty() {
        ""
    } else {
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{Attributes, Block, Inline, ListItem};
    use serde_yaml::{Mapping, Value};
    use std::collections::HashMap;

    #[test]
    fn parse_wraps_input_in_a_single_paragraph_text_node() {
        let result = parse("hello");

        assert!(result.errors.is_empty());
        assert!(result.document.frontmatter.is_none());
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "hello".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_splits_paragraphs_on_blank_lines() {
        let result = parse("first paragraph\n\nsecond paragraph");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "first paragraph".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "second paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_extracts_frontmatter_from_the_very_first_line() {
        let result = parse("---\ntitle: \"Winter Notes\"\nauthor: trailwriter\n---\n\nBody");
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");

        assert_eq!(
            frontmatter.raw,
            "title: \"Winter Notes\"\nauthor: trailwriter\n"
        );
        assert_eq!(
            frontmatter.fields.get("title"),
            Some(&Value::String("Winter Notes".to_string()))
        );
        assert_eq!(
            frontmatter.fields.get("author"),
            Some(&Value::String("trailwriter".to_string()))
        );
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Body".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_empty_frontmatter() {
        let result = parse("---\n---\n\nBody");
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");

        assert_eq!(frontmatter.raw, "");
        assert!(frontmatter.fields.is_empty());
        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "Body".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_non_first_line_frontmatter_marker_as_a_thematic_break() {
        let result = parse("\n---\n\nBody");

        assert!(result.document.frontmatter.is_none());
        assert_eq!(
            result.document.body,
            vec![
                Block::ThematicBreak,
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_supports_nested_yaml_values_in_frontmatter() {
        let result = parse(
            "---\nseries:\n  name: \"Northern Passages\"\n  part: 3\ndraft: true\n---\n\nBody",
        );
        let frontmatter = result.document.frontmatter.expect("expected frontmatter");
        let mut expected_series = Mapping::new();
        expected_series.insert(
            Value::String("name".to_string()),
            Value::String("Northern Passages".to_string()),
        );
        expected_series.insert(
            Value::String("part".to_string()),
            serde_yaml::to_value(3).expect("serializable integer"),
        );

        assert_eq!(
            frontmatter.fields.get("series"),
            Some(&Value::Mapping(expected_series))
        );
        assert_eq!(frontmatter.fields.get("draft"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_collapses_multiple_blank_lines_and_ignores_edge_whitespace() {
        let result = parse("\n \nfirst paragraph\n\n\t\n\nsecond paragraph\n  \n");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "first paragraph".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "second paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_keeps_multiline_paragraph_text_together() {
        let result = parse("line one  \nline two\nline three");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "line one  \nline two\nline three".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_headings_before_paragraph_fallback() {
        let result = parse("# Heading\nParagraph");

        assert_eq!(
            result.document.body,
            vec![
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text {
                        value: "Heading".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Paragraph".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_thematic_breaks_before_paragraph_fallback() {
        let result = parse("before\n\n---\n\nafter");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "before".to_string(),
                    }],
                    attrs: None,
                },
                Block::ThematicBreak,
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "after".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_fenced_code_blocks_before_paragraph_fallback() {
        let result = parse("before\n\n```rust\nfn main() {}\n```\n\nafter");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "before".to_string(),
                    }],
                    attrs: None,
                },
                Block::CodeBlock {
                    language: Some("rust".to_string()),
                    content: "fn main() {}".to_string(),
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "after".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_detects_blockquotes_before_paragraph_fallback() {
        let result = parse("> quoted\n\noutside");

        assert_eq!(
            result.document.body,
            vec![
                Block::BlockQuote {
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "quoted".to_string(),
                        }],
                        attrs: None,
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "outside".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_splits_blockquote_paragraphs_on_quoted_blank_lines() {
        let result = parse("> first paragraph\n>\n> second paragraph");

        assert_eq!(
            result.document.body,
            vec![Block::BlockQuote {
                content: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "first paragraph".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "second paragraph".to_string(),
                        }],
                        attrs: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_nested_blockquotes_by_recursively_parsing_stripped_content() {
        let result = parse("> outer\n>> inner\n>>> deepest");

        assert_eq!(
            result.document.body,
            vec![Block::BlockQuote {
                content: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "outer".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::BlockQuote {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "inner".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::BlockQuote {
                                content: vec![Block::Paragraph {
                                    content: vec![Inline::Text {
                                        value: "deepest".to_string(),
                                    }],
                                    attrs: None,
                                }],
                                attrs: None,
                            },
                        ],
                        attrs: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_keeps_fenced_code_block_content_raw() {
        let result = parse("```\n**bold**\n:note[keep literal]\n{~ not a comment ~}\n```");

        assert_eq!(
            result.document.body,
            vec![Block::CodeBlock {
                language: None,
                content: "**bold**\n:note[keep literal]\n{~ not a comment ~}".to_string(),
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_basic_block_directives_before_paragraph_fallback() {
        let result = parse("::aside\nInside the aside.\n::");

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: None,
                attrs: None,
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Inside the aside.".to_string(),
                    }],
                    attrs: None,
                }],
            }]
        );
    }

    #[test]
    fn parse_treats_blank_lines_inside_block_directives_as_body_content() {
        let result = parse("::aside\nFirst paragraph.\n\nSecond paragraph.\n::");

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: None,
                attrs: None,
                body: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "First paragraph.".to_string(),
                        }],
                        attrs: None,
                    },
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Second paragraph.".to_string(),
                        }],
                        attrs: None,
                    },
                ],
            }]
        );
    }

    #[test]
    fn parse_supports_block_directive_labels_and_attributes() {
        let result = parse("::aside[Author note]{#callout .highlight tone=\"quiet\"}\nBody\n::");
        let mut expected_pairs = HashMap::new();
        expected_pairs.insert("tone".to_string(), "quiet".to_string());

        assert_eq!(
            result.document.body,
            vec![Block::BlockDirective {
                name: "aside".to_string(),
                label: Some(vec![Inline::Text {
                    value: "Author note".to_string(),
                }]),
                attrs: Some(Attributes {
                    id: Some("callout".to_string()),
                    classes: vec!["highlight".to_string()],
                    pairs: expected_pairs,
                }),
                body: vec![Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "Body".to_string(),
                    }],
                    attrs: None,
                }],
            }]
        );
    }

    #[test]
    fn parse_rejects_block_directive_names_with_non_letter_non_hyphen_characters() {
        let result = parse("::note2\nBody\n::");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "::note2\nBody\n::".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_empty_fenced_code_blocks() {
        let result = parse("```\n```");

        assert_eq!(
            result.document.body,
            vec![Block::CodeBlock {
                language: None,
                content: String::new(),
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_accepts_all_thematic_break_markers_with_optional_spaces() {
        let result = parse("- - -\n***\n_ _ _ _");

        assert_eq!(
            result.document.body,
            vec![
                Block::ThematicBreak,
                Block::ThematicBreak,
                Block::ThematicBreak,
            ]
        );
    }

    #[test]
    fn parse_preserves_first_line_frontmatter_marker_as_non_thematic_break() {
        let result = parse("---");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "---".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_invalid_thematic_break_lines_as_paragraph_text() {
        let result = parse("--\n-*-");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "--\n-*-".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_invalid_hash_prefixes_as_paragraph_text() {
        let result = parse("#no-space\n\n####### Too many hashes");

        assert_eq!(
            result.document.body,
            vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "#no-space".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "####### Too many hashes".to_string(),
                    }],
                    attrs: None,
                },
            ]
        );
    }

    #[test]
    fn parse_ignores_the_required_leading_test_description_comment() {
        let result = parse("{~ test description ~}\n\nparagraph");

        assert_eq!(
            result.document.body,
            vec![Block::Paragraph {
                content: vec![Inline::Text {
                    value: "paragraph".to_string(),
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_returns_an_empty_body_for_whitespace_only_content() {
        let result = parse(" \n\t\n  ");

        assert!(result.document.body.is_empty());
    }

    #[test]
    fn parse_detects_unordered_lists_before_paragraph_fallback() {
        let result = parse("- pack rope\n- fill canteen");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "pack rope".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "fill canteen".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_checked_and_unchecked_task_list_items() {
        let result = parse("- [x] Reserve the campsite\n- [ ] Print the route map");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Reserve the campsite".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: Some(true),
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Print the route map".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: Some(false),
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_treats_indented_lines_as_unordered_list_item_continuations() {
        let result = parse(
            "- Camp briefing for the new arrivals.\n  Bring dry socks, a flashlight, and a map.\n\n  Check in before sunset.",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![
                        Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Camp briefing for the new arrivals.\nBring dry socks, a flashlight, and a map."
                                    .to_string(),
                            }],
                            attrs: None,
                        },
                        Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Check in before sunset.".to_string(),
                            }],
                            attrs: None,
                        },
                    ],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_parses_nested_blocks_inside_list_item_content() {
        let result = parse(
            "- Camp briefing for the new arrivals.\n  Bring dry socks and a flashlight.\n\n  > Check in before sunset.\n  > Keep your permit visible.",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![
                        Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Camp briefing for the new arrivals.\nBring dry socks and a flashlight."
                                    .to_string(),
                            }],
                            attrs: None,
                        },
                        Block::BlockQuote {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Check in before sunset.\nKeep your permit visible."
                                        .to_string(),
                                }],
                                attrs: None,
                            }],
                            attrs: None,
                        },
                    ],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_keeps_first_list_item_line_as_paragraph_text_when_it_starts_with_block_syntax() {
        let result = parse("- # not a heading");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![ListItem {
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "# not a heading".to_string(),
                        }],
                        attrs: None,
                    }],
                    checked: None,
                }],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_supports_nested_unordered_lists_with_lenient_indentation() {
        let result = parse(
            "- Weekend project\n  - Buy lumber\n    - Measure each board twice\n    - Mark the cut lines clearly\n- Workshop cleanup\n    - Sweep the sawdust",
        );

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Weekend project".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::List {
                                ordered: false,
                                items: vec![ListItem {
                                    content: vec![
                                        Block::Paragraph {
                                            content: vec![Inline::Text {
                                                value: "Buy lumber".to_string(),
                                            }],
                                            attrs: None,
                                        },
                                        Block::List {
                                            ordered: false,
                                            items: vec![
                                                ListItem {
                                                    content: vec![Block::Paragraph {
                                                        content: vec![Inline::Text {
                                                            value: "Measure each board twice"
                                                                .to_string(),
                                                        }],
                                                        attrs: None,
                                                    }],
                                                    checked: None,
                                                },
                                                ListItem {
                                                    content: vec![Block::Paragraph {
                                                        content: vec![Inline::Text {
                                                            value: "Mark the cut lines clearly"
                                                                .to_string(),
                                                        }],
                                                        attrs: None,
                                                    }],
                                                    checked: None,
                                                },
                                            ],
                                            attrs: None,
                                        },
                                    ],
                                    checked: None,
                                }],
                                attrs: None,
                            },
                        ],
                        checked: None,
                    },
                    ListItem {
                        content: vec![
                            Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Workshop cleanup".to_string(),
                                }],
                                attrs: None,
                            },
                            Block::List {
                                ordered: false,
                                items: vec![ListItem {
                                    content: vec![Block::Paragraph {
                                        content: vec![Inline::Text {
                                            value: "Sweep the sawdust".to_string(),
                                        }],
                                        attrs: None,
                                    }],
                                    checked: None,
                                }],
                                attrs: None,
                            },
                        ],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_detects_ordered_lists_before_paragraph_fallback() {
        let result = parse("1. Preheat the oven\n2. Chop the carrots");

        assert_eq!(
            result.document.body,
            vec![Block::List {
                ordered: true,
                items: vec![
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Preheat the oven".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                    ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Chop the carrots".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: None,
                    },
                ],
                attrs: None,
            }]
        );
    }

    #[test]
    fn parse_separates_ordered_lists_paragraphs_and_unordered_lists() {
        let result = parse(
            "1. Boil the water\n2. Add the tea leaves\n\nThe kettle can rest here.\n\n- Fold the camp blanket\n- Lock the storage bin",
        );

        assert_eq!(
            result.document.body,
            vec![
                Block::List {
                    ordered: true,
                    items: vec![
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Boil the water".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Add the tea leaves".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                    ],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text {
                        value: "The kettle can rest here.".to_string(),
                    }],
                    attrs: None,
                },
                Block::List {
                    ordered: false,
                    items: vec![
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Fold the camp blanket".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                        ListItem {
                            content: vec![Block::Paragraph {
                                content: vec![Inline::Text {
                                    value: "Lock the storage bin".to_string(),
                                }],
                                attrs: None,
                            }],
                            checked: None,
                        },
                    ],
                    attrs: None,
                },
            ]
        );
    }
}
