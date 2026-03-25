use crate::{Alignment, Block, TableCell};
use std::iter::Peekable;

use super::inline::parse_inlines;

pub(crate) fn table_from_lines<'a, I>(first_line: &'a str, lines: &mut Peekable<I>) -> Option<Block>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let headers = parse_table_row(first_line)?;
    let alignments = parse_alignment_row(lines.peek()?.1, headers.len())?;

    lines.next();

    let mut rows = Vec::new();

    while let Some((_, line)) = lines.peek().copied() {
        let Some(row) = parse_table_row_with_width(line, headers.len()) else {
            break;
        };

        lines.next();
        rows.push(row);
    }

    Some(Block::Table {
        headers,
        rows,
        alignments,
        attrs: None,
    })
}

fn parse_table_row(line: &str) -> Option<Vec<TableCell>> {
    let cells = split_table_row(line)?;

    Some(
        cells
            .into_iter()
            .map(|cell| TableCell {
                content: parse_inlines(cell.trim()),
            })
            .collect(),
    )
}

fn parse_table_row_with_width(line: &str, width: usize) -> Option<Vec<TableCell>> {
    let row = parse_table_row(line)?;
    (row.len() == width).then_some(row)
}

fn parse_alignment_row(line: &str, width: usize) -> Option<Vec<Alignment>> {
    let cells = split_table_row(line)?;

    if cells.len() != width {
        return None;
    }

    cells.into_iter().map(parse_alignment_cell).collect()
}

fn parse_alignment_cell(cell: &str) -> Option<Alignment> {
    let trimmed = cell.trim();

    if trimmed.is_empty() {
        return None;
    }

    let left_aligned = trimmed.starts_with(':');
    let right_aligned = trimmed.ends_with(':');
    let markers = trimmed.trim_matches(':');

    if markers.len() < 3 || !markers.bytes().all(|byte| byte == b'-') {
        return None;
    }

    Some(match (left_aligned, right_aligned) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        (false, false) => Alignment::None,
    })
}

fn split_table_row(line: &str) -> Option<Vec<&str>> {
    let trimmed = line.trim();

    if trimmed.len() < 2 || !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    Some(trimmed[1..trimmed.len() - 1].split('|').collect())
}

#[cfg(test)]
mod tests {
    use super::table_from_lines;
    use crate::{Alignment, Block, Inline};

    #[test]
    fn parses_pipe_tables_with_alignments_and_rows() {
        let mut lines = ["| :--- | :---: | ---: |", "| left | middle | 10 |"]
            .into_iter()
            .enumerate()
            .peekable();

        let block = table_from_lines("| Name | Status | Count |", &mut lines);

        assert_eq!(
            block,
            Some(Block::Table {
                headers: vec![
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "Name".to_string(),
                        }],
                    },
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "Status".to_string(),
                        }],
                    },
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "Count".to_string(),
                        }],
                    },
                ],
                rows: vec![vec![
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "left".to_string(),
                        }],
                    },
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "middle".to_string(),
                        }],
                    },
                    crate::TableCell {
                        content: vec![Inline::Text {
                            value: "10".to_string(),
                        }],
                    },
                ]],
                alignments: vec![Alignment::Left, Alignment::Center, Alignment::Right],
                attrs: None,
            })
        );
        assert!(lines.next().is_none());
    }

    #[test]
    fn parses_inline_content_in_cells() {
        let mut lines = [
            "| --- | --- |",
            "| **Preview** | Read the [guide](https://docs.etch-lang.dev/guide/tables) |",
        ]
        .into_iter()
        .enumerate()
        .peekable();

        let block = table_from_lines("| Feature | Notes |", &mut lines);
        let Block::Table { rows, .. } = block.expect("expected table") else {
            panic!("expected table block");
        };

        assert_eq!(
            rows[0],
            vec![
                crate::TableCell {
                    content: vec![Inline::Strong {
                        content: vec![Inline::Text {
                            value: "Preview".to_string(),
                        }],
                    }],
                },
                crate::TableCell {
                    content: vec![
                        Inline::Text {
                            value: "Read the ".to_string(),
                        },
                        Inline::Link {
                            url: "https://docs.etch-lang.dev/guide/tables".to_string(),
                            title: None,
                            content: vec![Inline::Text {
                                value: "guide".to_string(),
                            }],
                            attrs: None,
                        },
                    ],
                },
            ]
        );
    }

    #[test]
    fn rejects_tables_without_a_valid_separator_row() {
        let mut lines = ["| not a separator |", "| still text |"]
            .into_iter()
            .enumerate()
            .peekable();

        assert!(table_from_lines("| Header |", &mut lines).is_none());
    }
}
