use crate::{
    Alignment, Attributes, Block, DefinitionItem, Document, Frontmatter, Inline, ListItem,
    TableCell, math,
};
use std::collections::HashSet;

pub fn render_html(document: &Document) -> String {
    HtmlRenderer::default().render(document)
}

pub fn render_html_document(document: &Document) -> String {
    HtmlRenderer::default().render_document(document)
}

#[derive(Debug, Clone, Default)]
pub struct HtmlRenderer {
    headings: Vec<(u8, String, String)>,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&self, document: &Document) -> String {
        let mut renderer = self.clone();
        renderer.headings = collect_headings(&document.body);
        renderer.render_blocks(&document.body)
    }

    pub fn render_document(&self, document: &Document) -> String {
        let body = self.render(document);
        let title = document_title(document.frontmatter.as_ref())
            .unwrap_or_else(|| "Etch Document".to_string());

        format!(
            concat!(
                "<!DOCTYPE html>\n",
                "<html lang=\"en\">\n",
                "<head>\n",
                "<meta charset=\"utf-8\">\n",
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
                "<title>{}</title>\n",
                "</head>\n",
                "<body>\n",
                "{}\n",
                "</body>\n",
                "</html>"
            ),
            escape_html_text(&title),
            body
        )
    }

    fn render_blocks(&self, blocks: &[Block]) -> String {
        blocks
            .iter()
            .map(|block| self.render_block(block))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_block(&self, block: &Block) -> String {
        match block {
            Block::Paragraph { content, attrs } => {
                if attrs.is_none() && content.len() == 1 {
                    if let Some(html) = self.render_special_single_inline_paragraph(&content[0]) {
                        return html;
                    }
                }

                wrap_with_tag("p", attrs.as_ref(), &[], &[], &self.render_inlines(content))
            }
            Block::Heading {
                level,
                content,
                attrs,
            } => {
                let tag = format!("h{}", (*level).clamp(1, 6));
                let heading_attrs = heading_attributes(attrs.as_ref(), content);
                wrap_with_tag(
                    &tag,
                    heading_attrs.as_ref(),
                    &[],
                    &[],
                    &self.render_inlines(content),
                )
            }
            Block::CodeBlock {
                language,
                content,
                attrs,
            } => {
                let mut code_attrs = Vec::new();

                if let Some(language) = language {
                    code_attrs.push(("class", format!("language-{}", language)));
                }

                let code = render_voidless_tag("code", &code_attrs, &escape_html_text(content));

                wrap_with_tag("pre", attrs.as_ref(), &[], &[], &code)
            }
            Block::BlockQuote { content, attrs } => wrap_with_tag(
                "blockquote",
                attrs.as_ref(),
                &[],
                &[],
                &self.render_blocks(content),
            ),
            Block::List {
                ordered,
                items,
                attrs,
            } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let extra_classes = if items.iter().any(|item| item.checked.is_some()) {
                    vec!["task-list"]
                } else {
                    Vec::new()
                };
                let inner = items
                    .iter()
                    .map(|item| self.render_list_item(item))
                    .collect::<Vec<_>>()
                    .join("");

                wrap_with_tag(tag, attrs.as_ref(), &[], &extra_classes, &inner)
            }
            Block::Table {
                headers,
                rows,
                alignments,
                attrs,
            } => self.render_table(headers, rows, alignments, attrs.as_ref()),
            Block::ThematicBreak => "<hr>".to_string(),
            Block::BlockDirective {
                directive_id,
                name,
                label,
                raw_label,
                attrs,
                raw_body,
                body,
                ..
            } => {
                if name == "math" {
                    return math::latex_to_mathml(raw_body, true);
                }

                match name.as_str() {
                    "note" => self.render_note_block(attrs.as_ref(), label.as_deref(), body),
                    "aside" => self.render_aside_block(attrs.as_ref(), label.as_deref(), body),
                    "figure" => self.render_figure_block(attrs.as_ref(), label.as_deref(), body),
                    "details" => self.render_details_block(attrs.as_ref(), label.as_deref(), body),
                    "toc" => self.render_toc(attrs.as_ref()),
                    "pagebreak" => self.render_pagebreak(attrs.as_ref()),
                    _ => self.render_directive(
                        "div",
                        "block",
                        *directive_id,
                        name,
                        raw_label.as_deref(),
                        raw_body,
                        label.as_deref(),
                        attrs.as_ref(),
                        body,
                    ),
                }
            }
            Block::ContainerDirective {
                directive_id,
                name,
                label,
                raw_label,
                attrs,
                raw_body,
                body,
                ..
            } => {
                if name == "math" {
                    return math::latex_to_mathml(raw_body, true);
                }

                match name.as_str() {
                    "section" => self.render_section_block(attrs.as_ref(), label.as_deref(), body),
                    "chapter" => self.render_chapter_block(attrs.as_ref(), label.as_deref(), body),
                    "columns" => self.render_columns_block(attrs.as_ref(), label.as_deref(), body),
                    "column" => self.render_column_block(attrs.as_ref(), label.as_deref(), body),
                    _ => self.render_directive(
                        "section",
                        "container",
                        *directive_id,
                        name,
                        raw_label.as_deref(),
                        raw_body,
                        label.as_deref(),
                        attrs.as_ref(),
                        body,
                    ),
                }
            }
            Block::FootnoteDefinition { label, content } => {
                let reference = wrap_with_tag(
                    "a",
                    None,
                    &[("href", format!("#{}", footnote_ref_id(label)))],
                    &[],
                    &escape_html_text(label),
                );
                let label_html = wrap_with_tag(
                    "p",
                    None,
                    &[("class", "footnote-label".to_string())],
                    &[],
                    &wrap_with_tag("sup", None, &[], &[], &reference),
                );
                let blocks = self.render_blocks(content);
                let inner = if blocks.is_empty() {
                    label_html
                } else {
                    format!("{label_html}\n{blocks}")
                };

                wrap_with_tag(
                    "div",
                    None,
                    &[
                        ("id", footnote_id(label)),
                        ("class", "footnote".to_string()),
                        ("data-footnote-label", label.clone()),
                    ],
                    &[],
                    &inner,
                )
            }
            Block::DefinitionList { items, attrs } => {
                let inner = items
                    .iter()
                    .map(|item| self.render_definition_item(item))
                    .collect::<Vec<_>>()
                    .join("");

                wrap_with_tag("dl", attrs.as_ref(), &[], &[], &inner)
            }
        }
    }

    fn render_list_item(&self, item: &ListItem) -> String {
        let mut extra_attrs = Vec::new();
        let mut inner = String::new();
        let mut extra_classes = Vec::new();

        if let Some(checked) = item.checked {
            extra_attrs.push(("data-task", "true".to_string()));
            extra_attrs.push(("data-checked", checked.to_string()));
            extra_classes.push("task-list-item");
            inner.push_str("<div class=\"task-list-item__body\">");
            inner.push_str("<input class=\"task-list-item__checkbox\" type=\"checkbox\" disabled");

            if checked {
                inner.push_str(" checked");
            }

            inner.push('>');
            inner.push_str("<div class=\"task-list-item__content\">");
            inner.push_str(&self.render_blocks(&item.content));
            inner.push_str("</div></div>");
        } else {
            inner.push_str(&self.render_blocks(&item.content));
        }

        wrap_with_tag("li", None, &extra_attrs, &extra_classes, &inner)
    }

    fn render_table(
        &self,
        headers: &[TableCell],
        rows: &[Vec<TableCell>],
        alignments: &[Alignment],
        attrs: Option<&Attributes>,
    ) -> String {
        let head_cells = headers
            .iter()
            .enumerate()
            .map(|(index, cell)| self.render_table_cell("th", cell, alignments.get(index)))
            .collect::<Vec<_>>()
            .join("");
        let thead = format!("<thead><tr>{}</tr></thead>", head_cells);

        let body_rows = rows
            .iter()
            .map(|row| {
                let cells = row
                    .iter()
                    .enumerate()
                    .map(|(index, cell)| self.render_table_cell("td", cell, alignments.get(index)))
                    .collect::<Vec<_>>()
                    .join("");

                format!("<tr>{}</tr>", cells)
            })
            .collect::<Vec<_>>()
            .join("");
        let tbody = format!("<tbody>{}</tbody>", body_rows);

        wrap_with_tag("table", attrs, &[], &[], &format!("{}{}", thead, tbody))
    }

    fn render_table_cell(
        &self,
        tag: &str,
        cell: &TableCell,
        alignment: Option<&Alignment>,
    ) -> String {
        let mut extra_attrs = Vec::new();

        if let Some(style) = alignment_style(alignment) {
            extra_attrs.push(("style", style.to_string()));
        }

        wrap_with_tag(
            tag,
            None,
            &extra_attrs,
            &[],
            &self.render_inlines(&cell.content),
        )
    }

    fn render_definition_item(&self, item: &DefinitionItem) -> String {
        let term = wrap_with_tag("dt", None, &[], &[], &self.render_inlines(&item.term));
        let definitions = item
            .definitions
            .iter()
            .map(|definition| wrap_with_tag("dd", None, &[], &[], &self.render_blocks(definition)))
            .collect::<Vec<_>>()
            .join("");

        format!("{}{}", term, definitions)
    }

    fn render_note_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        const VALID_TYPES: &[&str] = &["info", "tip", "warning", "caution", "danger"];

        let note_type = attrs
            .and_then(|attrs| attrs.pairs.get("type"))
            .filter(|value| VALID_TYPES.contains(&value.as_str()))
            .map(String::as_str);
        let modifier_class = note_type.map(|note_type| format!("note--{note_type}"));
        let mut classes = vec!["note"];

        if let Some(modifier_class) = modifier_class.as_deref() {
            classes.push(modifier_class);
        }

        let mut inner = String::new();

        if let Some(label) = label {
            inner.push_str(&wrap_with_tag(
                "p",
                None,
                &[("class", "note-label".to_string())],
                &[],
                &self.render_inlines(label),
            ));

            if !body.is_empty() {
                inner.push('\n');
            }
        }

        inner.push_str(&self.render_blocks(body));

        wrap_with_tag(
            "aside",
            attrs,
            &[("role", "note".to_string())],
            &classes,
            &inner,
        )
    }

    fn render_aside_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        wrap_with_tag(
            "aside",
            attrs,
            &[],
            &["aside"],
            &self.render_labeled_blocks(label, "directive-label", body),
        )
    }

    fn render_figure_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        let (attrs, caption) = take_attr(attrs, "caption");
        let mut inner = self.render_blocks(body);
        let caption = label.map(|label| self.render_inlines(label)).or(caption);

        if let Some(caption) = caption {
            if !inner.is_empty() {
                inner.push('\n');
            }

            inner.push_str(&wrap_with_tag("figcaption", None, &[], &[], &caption));
        }

        wrap_with_tag("figure", attrs.as_ref(), &[], &[], &inner)
    }

    fn render_details_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        let mut inner = String::new();

        if let Some(label) = label {
            inner.push_str(&wrap_with_tag(
                "summary",
                None,
                &[],
                &[],
                &self.render_inlines(label),
            ));
        } else {
            inner.push_str(&wrap_with_tag("summary", None, &[], &[], "Details"));
        }

        if !body.is_empty() {
            inner.push('\n');
        }

        inner.push_str(&wrap_with_tag(
            "div",
            None,
            &[("class", "details-content".to_string())],
            &[],
            &self.render_blocks(body),
        ));

        wrap_with_tag("details", attrs, &[], &["details"], &inner)
    }

    fn render_toc(&self, attrs: Option<&Attributes>) -> String {
        let items = self
            .headings
            .iter()
            .map(|(_, text, slug)| {
                format!(
                    "<li><a href=\"#{}\">{}</a></li>",
                    escape_html_attr(slug),
                    escape_html_text(text),
                )
            })
            .collect::<Vec<_>>()
            .join("");

        wrap_with_tag(
            "nav",
            attrs,
            &[("aria-label", "Table of contents".to_string())],
            &["toc"],
            &wrap_with_tag("ol", None, &[], &[], &items),
        )
    }

    fn render_pagebreak(&self, attrs: Option<&Attributes>) -> String {
        wrap_with_tag("div", attrs, &[], &["page-break"], "")
    }

    fn render_section_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        let mut extra_attrs = Vec::new();

        if let Some(title) = attrs.and_then(|attrs| attrs.pairs.get("title")) {
            extra_attrs.push(("aria-label", title.clone()));
        }

        wrap_with_tag(
            "section",
            attrs,
            &extra_attrs,
            &[],
            &self.render_labeled_blocks(label, "directive-label", body),
        )
    }

    fn render_chapter_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        let mut extra_attrs = Vec::new();

        if let Some(title) = attrs.and_then(|attrs| attrs.pairs.get("title")) {
            extra_attrs.push(("aria-label", title.clone()));
        }

        wrap_with_tag(
            "section",
            attrs,
            &extra_attrs,
            &["chapter"],
            &self.render_labeled_blocks(label, "directive-label", body),
        )
    }

    fn render_columns_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        let count = attrs
            .and_then(|attrs| attrs.pairs.get("count"))
            .map(String::as_str)
            .unwrap_or("2");
        let gap = attrs.and_then(|attrs| attrs.pairs.get("gap"));
        let mut style = format!("--columns-count: {count}");

        if let Some(gap) = gap {
            style.push_str(&format!("; --columns-gap: {gap}"));
        }

        wrap_with_tag(
            "div",
            attrs,
            &[("style", style)],
            &["columns"],
            &self.render_labeled_blocks(label, "directive-label", body),
        )
    }

    fn render_column_block(
        &self,
        attrs: Option<&Attributes>,
        label: Option<&[Inline]>,
        body: &[Block],
    ) -> String {
        wrap_with_tag(
            "div",
            attrs,
            &[],
            &["column"],
            &self.render_labeled_blocks(label, "directive-label", body),
        )
    }

    fn render_labeled_blocks(
        &self,
        label: Option<&[Inline]>,
        label_class: &str,
        body: &[Block],
    ) -> String {
        let mut inner = String::new();

        if let Some(label) = label {
            inner.push_str(&wrap_with_tag(
                "p",
                None,
                &[("class", label_class.to_string())],
                &[],
                &self.render_inlines(label),
            ));

            if !body.is_empty() {
                inner.push('\n');
            }
        }

        inner.push_str(&self.render_blocks(body));
        inner
    }

    fn render_special_single_inline_paragraph(&self, inline: &Inline) -> Option<String> {
        let Inline::InlineDirective {
            name,
            content,
            attrs,
            ..
        } = inline
        else {
            return None;
        };

        if attrs.is_some() || content.is_some() {
            return None;
        }

        match name.as_str() {
            "pagebreak" => Some(self.render_pagebreak(None)),
            "toc" => Some(self.render_toc(None)),
            _ => None,
        }
    }

    fn render_directive(
        &self,
        tag: &str,
        kind: &str,
        directive_id: u64,
        name: &str,
        raw_label: Option<&str>,
        raw_content: &str,
        label: Option<&[Inline]>,
        attrs: Option<&Attributes>,
        body: &[Block],
    ) -> String {
        let mut inner = String::new();

        if let Some(label) = label {
            inner.push_str(&wrap_with_tag(
                "p",
                None,
                &[("class", "directive-label".to_string())],
                &[],
                &self.render_inlines(label),
            ));

            if !body.is_empty() {
                inner.push('\n');
            }
        }

        inner.push_str(&self.render_blocks(body));

        let mut extra_attrs = vec![
            ("data-etch-directive", name.to_string()),
            ("data-etch-kind", kind.to_string()),
            ("data-etch-id", directive_id.to_string()),
            ("data-etch-content", raw_content.to_string()),
        ];

        if let Some(raw_label) = raw_label {
            extra_attrs.push(("data-etch-label", raw_label.to_string()));
        }

        if let Some(attrs) = attrs {
            extra_attrs.push(("data-etch-attrs", serialize_attrs(attrs)));
        }

        wrap_with_tag(tag, None, &extra_attrs, &[], &inner)
    }

    fn render_inlines(&self, inlines: &[Inline]) -> String {
        let mut html = String::new();

        for inline in inlines {
            match inline {
                Inline::Text { value } => html.push_str(&escape_html_text(value)),
                Inline::Emphasis { content } => {
                    html.push_str(&wrap_with_tag(
                        "em",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Strong { content } => {
                    html.push_str(&wrap_with_tag(
                        "strong",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Strikethrough { content } => {
                    html.push_str(&wrap_with_tag(
                        "del",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::InlineCode { value } => {
                    html.push_str(&wrap_with_tag(
                        "code",
                        None,
                        &[],
                        &[],
                        &escape_html_text(value),
                    ));
                }
                Inline::Superscript { content } => {
                    html.push_str(&wrap_with_tag(
                        "sup",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Subscript { content } => {
                    html.push_str(&wrap_with_tag(
                        "sub",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Highlight { content } => {
                    html.push_str(&wrap_with_tag(
                        "mark",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Spoiler { content } => {
                    let inner_content = self.render_inlines(content);
                    let checkbox = render_void_tag(
                        "input",
                        None,
                        &[
                            ("type", "checkbox".to_string()),
                            ("class", "spoiler-toggle".to_string()),
                        ],
                        &[],
                    );
                    let span = wrap_with_tag(
                        "span",
                        None,
                        &[],
                        &["spoiler-content"],
                        &inner_content,
                    );
                    html.push_str(&wrap_with_tag(
                        "label",
                        None,
                        &[],
                        &["spoiler"],
                        &format!("{checkbox}{span}"),
                    ));
                }
                Inline::Insert { content } => {
                    html.push_str(&wrap_with_tag(
                        "ins",
                        None,
                        &[],
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Link {
                    url,
                    title,
                    content,
                    attrs,
                } => {
                    let mut extra_attrs = vec![("href", url.clone())];

                    if let Some(title) = title {
                        extra_attrs.push(("title", title.clone()));
                    }

                    html.push_str(&wrap_with_tag(
                        "a",
                        attrs.as_ref(),
                        &extra_attrs,
                        &[],
                        &self.render_inlines(content),
                    ));
                }
                Inline::Image {
                    url,
                    alt,
                    title,
                    attrs,
                } => {
                    let mut extra_attrs = vec![("src", url.clone()), ("alt", alt.clone())];

                    if let Some(title) = title {
                        extra_attrs.push(("title", title.clone()));
                    }

                    html.push_str(&render_void_tag("img", attrs.as_ref(), &extra_attrs, &[]));
                }
                Inline::AutoLink { url } => {
                    html.push_str(&wrap_with_tag(
                        "a",
                        None,
                        &[("href", url.clone())],
                        &[],
                        &escape_html_text(url),
                    ));
                }
                Inline::InlineDirective {
                    directive_id,
                    name,
                    content,
                    raw_content,
                    attrs,
                    ..
                } => {
                    if name == "math" {
                        html.push_str(&math::latex_to_mathml(
                            raw_content.as_deref().unwrap_or_default(),
                            false,
                        ));
                        continue;
                    }

                    if name == "pagebreak" {
                        html.push_str(&wrap_with_tag(
                            "span",
                            attrs.as_ref(),
                            &[("aria-hidden", "true".to_string())],
                            &["page-break"],
                            "",
                        ));
                        continue;
                    }

                    if name == "toc" {
                        html.push_str(&wrap_with_tag(
                            "span",
                            attrs.as_ref(),
                            &[
                                ("role", "navigation".to_string()),
                                ("aria-label", "Table of contents".to_string()),
                            ],
                            &["toc"],
                            "",
                        ));
                        continue;
                    }

                    if name == "abbr" {
                        let (attrs, title) = take_attr(attrs.as_ref(), "title");
                        let mut extra_attrs = Vec::new();

                        if let Some(title) = title {
                            extra_attrs.push(("title", title));
                        }

                        html.push_str(&wrap_with_tag(
                            "abbr",
                            attrs.as_ref(),
                            &extra_attrs,
                            &[],
                            &self.render_inlines(content.as_deref().unwrap_or(&[])),
                        ));
                        continue;
                    }

                    if name == "kbd" {
                        let raw = raw_content.as_deref().unwrap_or_default();
                        let keys = raw.split('+').map(str::trim).collect::<Vec<_>>();

                        if keys.len() > 1 {
                            html.push_str(
                                &keys
                                    .iter()
                                    .map(|key| format!("<kbd>{}</kbd>", escape_html_text(key)))
                                    .collect::<Vec<_>>()
                                    .join("+"),
                            );
                        } else {
                            html.push_str(&wrap_with_tag(
                                "kbd",
                                attrs.as_ref(),
                                &[],
                                &[],
                                &escape_html_text(raw.trim()),
                            ));
                        }
                        continue;
                    }

                    if name == "cite" {
                        html.push_str(&wrap_with_tag(
                            "cite",
                            attrs.as_ref(),
                            &[],
                            &[],
                            &self.render_inlines(content.as_deref().unwrap_or(&[])),
                        ));
                        continue;
                    }

                    let rendered_content = content
                        .as_deref()
                        .map(|content| self.render_inlines(content))
                        .unwrap_or_default();

                    let mut extra_attrs = vec![
                        ("data-etch-directive", name.to_string()),
                        ("data-etch-kind", "inline".to_string()),
                        ("data-etch-id", directive_id.to_string()),
                        (
                            "data-etch-content",
                            raw_content.as_deref().unwrap_or_default().to_string(),
                        ),
                    ];

                    if let Some(attrs) = attrs.as_ref() {
                        extra_attrs.push(("data-etch-attrs", serialize_attrs(attrs)));
                    }

                    html.push_str(&wrap_with_tag(
                        "span",
                        None,
                        &extra_attrs,
                        &[],
                        &rendered_content,
                    ));
                }
                Inline::FootnoteReference { label } => {
                    let reference = wrap_with_tag(
                        "a",
                        None,
                        &[
                            ("id", footnote_ref_id(label)),
                            ("href", format!("#{}", footnote_id(label))),
                        ],
                        &[],
                        &escape_html_text(label),
                    );

                    html.push_str(&wrap_with_tag("sup", None, &[], &[], &reference));
                }
                Inline::SoftBreak => html.push('\n'),
                Inline::HardBreak => html.push_str("<br>"),
            }
        }

        html
    }
}

fn take_attr(attrs: Option<&Attributes>, key: &str) -> (Option<Attributes>, Option<String>) {
    let mut cloned = attrs.cloned();
    let value = cloned.as_mut().and_then(|attrs| attrs.pairs.remove(key));
    (cloned, value)
}

fn heading_attributes(attrs: Option<&Attributes>, content: &[Inline]) -> Option<Attributes> {
    let mut cloned = attrs.cloned();
    let slug = heading_slug(content);

    match (&mut cloned, slug) {
        (Some(attrs), Some(slug)) if attrs.id.is_none() => {
            attrs.id = Some(slug);
            cloned
        }
        (None, Some(slug)) => Some(Attributes {
            id: Some(slug),
            classes: Vec::new(),
            pairs: std::collections::HashMap::new(),
        }),
        _ => cloned,
    }
}

fn heading_slug(content: &[Inline]) -> Option<String> {
    let mut text = String::new();
    collect_inline_text(content, &mut text);
    let slug = slugify(&text);

    if slug.is_empty() { None } else { Some(slug) }
}

fn collect_inline_text(inlines: &[Inline], text: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text { value } | Inline::InlineCode { value } => text.push_str(value),
            Inline::Emphasis { content }
            | Inline::Strong { content }
            | Inline::Strikethrough { content }
            | Inline::Superscript { content }
            | Inline::Subscript { content }
            | Inline::Highlight { content }
            | Inline::Insert { content }
            | Inline::Spoiler { content } => collect_inline_text(content, text),
            Inline::Link { content, .. } => collect_inline_text(content, text),
            Inline::Image { alt, .. } => text.push_str(alt),
            Inline::AutoLink { url } => text.push_str(url),
            Inline::InlineDirective {
                content,
                raw_content,
                ..
            } => {
                if let Some(content) = content {
                    collect_inline_text(content, text);
                } else if let Some(raw_content) = raw_content {
                    text.push_str(raw_content);
                }
            }
            Inline::FootnoteReference { label } => text.push_str(label),
            Inline::SoftBreak | Inline::HardBreak => text.push(' '),
        }
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !slug.is_empty() && !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

fn collect_headings(blocks: &[Block]) -> Vec<(u8, String, String)> {
    let mut headings = Vec::new();
    collect_headings_into(blocks, &mut headings);
    headings
}

fn collect_headings_into(blocks: &[Block], headings: &mut Vec<(u8, String, String)>) {
    for block in blocks {
        match block {
            Block::Heading {
                level,
                content,
                attrs,
            } => {
                let text = heading_text(content);
                let id = attrs
                    .as_ref()
                    .and_then(|attrs| attrs.id.clone())
                    .or_else(|| heading_slug(content))
                    .unwrap_or_default();

                headings.push((*level, text, id));
            }
            Block::BlockQuote { content, .. } => collect_headings_into(content, headings),
            Block::List { items, .. } => {
                for item in items {
                    collect_headings_into(&item.content, headings);
                }
            }
            Block::BlockDirective { body, .. } | Block::ContainerDirective { body, .. } => {
                collect_headings_into(body, headings);
            }
            Block::FootnoteDefinition { content, .. } => {
                collect_headings_into(content, headings);
            }
            Block::DefinitionList { items, .. } => {
                for item in items {
                    for definition in &item.definitions {
                        collect_headings_into(definition, headings);
                    }
                }
            }
            Block::Paragraph { .. }
            | Block::CodeBlock { .. }
            | Block::Table { .. }
            | Block::ThematicBreak => {}
        }
    }
}

fn heading_text(content: &[Inline]) -> String {
    let mut text = String::new();
    collect_inline_text(content, &mut text);
    text
}

fn render_voidless_tag(tag: &str, attrs: &[(&str, String)], inner: &str) -> String {
    let mut html = String::new();
    html.push('<');
    html.push_str(tag);
    push_extra_attributes(&mut html, attrs, &mut HashSet::new());
    html.push('>');
    html.push_str(inner);
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
    html
}

fn wrap_with_tag(
    tag: &str,
    attrs: Option<&Attributes>,
    extra_attrs: &[(&str, String)],
    extra_classes: &[&str],
    inner: &str,
) -> String {
    let mut html = String::new();
    html.push('<');
    html.push_str(tag);
    push_attributes(&mut html, attrs, extra_attrs, extra_classes);
    html.push('>');
    html.push_str(inner);
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
    html
}

fn render_void_tag(
    tag: &str,
    attrs: Option<&Attributes>,
    extra_attrs: &[(&str, String)],
    extra_classes: &[&str],
) -> String {
    let mut html = String::new();
    html.push('<');
    html.push_str(tag);
    push_attributes(&mut html, attrs, extra_attrs, extra_classes);
    html.push('>');
    html
}

fn push_attributes(
    html: &mut String,
    attrs: Option<&Attributes>,
    extra_attrs: &[(&str, String)],
    extra_classes: &[&str],
) {
    let mut used_names = HashSet::new();

    if let Some(attrs) = attrs {
        if let Some(id) = &attrs.id {
            push_attr(html, "id", id);
            used_names.insert("id".to_string());
        }

        let mut classes = attrs.classes.clone();
        classes.extend(extra_classes.iter().map(|class| (*class).to_string()));

        if !classes.is_empty() {
            push_attr(html, "class", &classes.join(" "));
            used_names.insert("class".to_string());
        }
    } else if !extra_classes.is_empty() {
        push_attr(html, "class", &extra_classes.join(" "));
        used_names.insert("class".to_string());
    }

    push_extra_attributes(html, extra_attrs, &mut used_names);

    if let Some(attrs) = attrs {
        let mut data_attrs = attrs.pairs.iter().collect::<Vec<_>>();
        data_attrs.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

        for (key, value) in data_attrs {
            let name = normalize_data_attribute_name(key);

            if used_names.insert(name.clone()) {
                push_attr(html, &name, value);
            }
        }
    }
}

fn push_extra_attributes(
    html: &mut String,
    extra_attrs: &[(&str, String)],
    used_names: &mut HashSet<String>,
) {
    for (name, value) in extra_attrs {
        if used_names.insert((*name).to_string()) {
            push_attr(html, name, value);
        }
    }
}

fn push_attr(html: &mut String, name: &str, value: &str) {
    html.push(' ');
    html.push_str(name);
    html.push_str("=\"");
    html.push_str(&escape_html_attr(value));
    html.push('"');
}

fn alignment_style(alignment: Option<&Alignment>) -> Option<&'static str> {
    match alignment {
        Some(Alignment::Left) => Some("text-align: left;"),
        Some(Alignment::Center) => Some("text-align: center;"),
        Some(Alignment::Right) => Some("text-align: right;"),
        _ => None,
    }
}

fn footnote_id(label: &str) -> String {
    format!("fn-{}", label)
}

fn footnote_ref_id(label: &str) -> String {
    format!("fnref-{}", label)
}

fn document_title(frontmatter: Option<&Frontmatter>) -> Option<String> {
    let frontmatter = frontmatter?;
    let value = frontmatter.fields.get("title")?;

    value.as_title_string()
}

fn serialize_attrs(attrs: &Attributes) -> String {
    let mut fields = Vec::new();

    if let Some(id) = &attrs.id {
        fields.push(("id".to_string(), id.clone()));
    }

    if !attrs.classes.is_empty() {
        fields.push(("class".to_string(), attrs.classes.join(" ")));
    }

    let mut pairs = attrs.pairs.iter().collect::<Vec<_>>();
    pairs.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    fields.extend(
        pairs
            .into_iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );

    let body = fields
        .into_iter()
        .map(|(key, value)| format!("\"{}\":\"{}\"", escape_json(&key), escape_json(&value)))
        .collect::<Vec<_>>()
        .join(",");

    format!("{{{body}}}")
}

fn normalize_data_attribute_name(name: &str) -> String {
    if name.starts_with("data-") {
        name.to_string()
    } else {
        format!("data-{}", name)
    }
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attr(value: &str) -> String {
    escape_html_text(value).replace('"', "&quot;")
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();

    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::{render_html, render_html_document};
    use crate::{
        Alignment, Attributes, Block, Document, Frontmatter, FrontmatterValue, Inline, ListItem,
        TableCell, parse,
    };
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn renders_core_nodes_to_html() {
        let document = Document {
            frontmatter: Some(Frontmatter {
                raw: "title: Example".to_string(),
                fields: BTreeMap::from([(
                    "title".to_string(),
                    FrontmatterValue::String("Example".to_string()),
                )]),
            }),
            body: vec![
                Block::Heading {
                    level: 2,
                    content: vec![Inline::Text {
                        value: "Title".to_string(),
                    }],
                    attrs: Some(Attributes {
                        id: Some("hero".to_string()),
                        classes: vec!["display".to_string()],
                        pairs: HashMap::from([("tone".to_string(), "warm".to_string())]),
                    }),
                },
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text {
                        value: "Auto Heading".to_string(),
                    }],
                    attrs: None,
                },
                Block::Paragraph {
                    content: vec![
                        Inline::Text {
                            value: "Hello ".to_string(),
                        },
                        Inline::Emphasis {
                            content: vec![Inline::Text {
                                value: "world".to_string(),
                            }],
                        },
                        Inline::Text {
                            value: " and ".to_string(),
                        },
                        Inline::Strong {
                            content: vec![Inline::Text {
                                value: "friends".to_string(),
                            }],
                        },
                        Inline::Text {
                            value: ".".to_string(),
                        },
                    ],
                    attrs: None,
                },
                Block::CodeBlock {
                    language: Some("rust".to_string()),
                    content: "fn main() {}".to_string(),
                    attrs: None,
                },
                Block::List {
                    ordered: false,
                    items: vec![ListItem {
                        content: vec![Block::Paragraph {
                            content: vec![Inline::Text {
                                value: "Todo".to_string(),
                            }],
                            attrs: None,
                        }],
                        checked: Some(true),
                    }],
                    attrs: None,
                },
                Block::Table {
                    headers: vec![TableCell {
                        content: vec![Inline::Text {
                            value: "Name".to_string(),
                        }],
                    }],
                    rows: vec![vec![TableCell {
                        content: vec![Inline::Link {
                            url: "https://example.com".to_string(),
                            title: Some("Go".to_string()),
                            content: vec![Inline::Text {
                                value: "Etch".to_string(),
                            }],
                            attrs: None,
                        }],
                    }]],
                    alignments: vec![Alignment::Center],
                    attrs: None,
                },
                Block::BlockDirective {
                    directive_id: 1,
                    span: crate::SourceSpan {
                        start: crate::SourcePosition { line: 1, column: 1 },
                        end: crate::SourcePosition { line: 1, column: 1 },
                    },
                    name: "aside".to_string(),
                    label: Some(vec![Inline::Text {
                        value: "Note".to_string(),
                    }]),
                    raw_label: Some("Note".to_string()),
                    attrs: None,
                    raw_body: "Directive body".to_string(),
                    body: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Directive body".to_string(),
                        }],
                        attrs: None,
                    }],
                },
                Block::ContainerDirective {
                    directive_id: 2,
                    span: crate::SourceSpan {
                        start: crate::SourcePosition { line: 1, column: 1 },
                        end: crate::SourcePosition { line: 1, column: 1 },
                    },
                    name: "chapter".to_string(),
                    label: None,
                    raw_label: None,
                    attrs: None,
                    raw_body: "[^a]".to_string(),
                    body: vec![Block::Paragraph {
                        content: vec![Inline::FootnoteReference {
                            label: "a".to_string(),
                        }],
                        attrs: None,
                    }],
                    named_close: true,
                },
                Block::FootnoteDefinition {
                    label: "a".to_string(),
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Footnote".to_string(),
                        }],
                        attrs: None,
                    }],
                },
            ],
        };

        let fragment = render_html(&document);

        assert_eq!(
            fragment,
            concat!(
                "<h2 id=\"hero\" class=\"display\" data-tone=\"warm\">Title</h2>\n",
                "<h1 id=\"auto-heading\">Auto Heading</h1>\n",
                "<p>Hello <em>world</em> and <strong>friends</strong>.</p>\n",
                "<pre><code class=\"language-rust\">fn main() {}</code></pre>\n",
                "<ul class=\"task-list\"><li class=\"task-list-item\" data-task=\"true\" data-checked=\"true\"><div class=\"task-list-item__body\"><input class=\"task-list-item__checkbox\" type=\"checkbox\" disabled checked><div class=\"task-list-item__content\"><p>Todo</p></div></div></li></ul>\n",
                "<table><thead><tr><th style=\"text-align: center;\">Name</th></tr></thead><tbody><tr><td style=\"text-align: center;\"><a href=\"https://example.com\" title=\"Go\">Etch</a></td></tr></tbody></table>\n",
                "<aside class=\"aside\"><p class=\"directive-label\">Note</p>\n<p>Directive body</p></aside>\n",
                "<section class=\"chapter\"><p><sup><a id=\"fnref-a\" href=\"#fn-a\">a</a></sup></p></section>\n",
                "<div id=\"fn-a\" class=\"footnote\" data-footnote-label=\"a\"><p class=\"footnote-label\"><sup><a href=\"#fnref-a\">a</a></sup></p>\n<p>Footnote</p></div>"
            )
        );

        let full_document = render_html_document(&document);
        assert!(full_document.contains("<title>Example</title>"));
        assert!(full_document.contains(fragment.as_str()));
    }

    #[test]
    fn renders_embers_in_the_snow_fixture() {
        let input = include_str!("../../../../tests/corpus/integration/embers-in-the-snow.etch");
        let result = parse(input);
        assert!(
            result.errors.is_empty(),
            "expected fixture to parse cleanly"
        );

        let html = render_html(&result.document);

        // Title heading with auto-id
        assert!(html.contains("<h1 id=\"embers-in-the-snow\">Embers in the Snow</h1>"));

        // Dedication rendered as aside (was ::dedication, now ::aside)
        assert!(html.contains("<aside class=\"aside\">"));
        assert!(html.contains("stayed up too late reading under the covers"));

        // Content note rendered as note with type=caution
        assert!(html.contains("class=\"note note--caution\""));
        assert!(html.contains("role=\"note\""));
        assert!(html.contains("<p class=\"note-label\">Content note</p>"));
        assert!(html.contains("Themes of nostalgia"));

        // Table of contents
        assert!(html.contains("<nav class=\"toc\""));

        // Chapter container
        assert!(html.contains("<section class=\"chapter\""));
        assert!(html.contains("aria-label=\"The First Snow\""));

        // Highlight marks
        assert!(html.contains("<mark>absolute</mark>"));

        // Blockquote with attribution
        assert!(html.contains(
            "<blockquote><p><em>\"I'll come back when the embers remember how to burn.\"</em></p>"
        ));

        // Inline math
        assert!(html.contains("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">"));

        // Inline spoiler
        assert!(html.contains("<label class=\"spoiler\">"));
        assert!(html.contains("<span class=\"spoiler-content\">a small wooden box, half-buried in ash</span>"));

        // Hard line breaks
        assert!(
            html.contains(
                "Her address was simple:<br>42 Northwind Road<br>The Village at the Edge"
            )
        );

        // Footnotes
        assert!(html.contains("<sup><a id=\"fnref-1\" href=\"#fn-1\">1</a></sup>"));
        assert!(html.contains("<div id=\"fn-1\" class=\"footnote\" data-footnote-label=\"1\">"));
        assert!(
            html.contains("<p class=\"footnote-label\"><sup><a href=\"#fnref-1\">1</a></sup></p>")
        );
    }

    #[test]
    fn renders_note_directive() {
        let html = render_html(&parse("::note[Important]\nDo not forget this.\n::").document);
        assert!(html.contains("<aside class=\"note\""));
        assert!(html.contains("role=\"note\""));
        assert!(html.contains("<p class=\"note-label\">Important</p>"));
        assert!(html.contains("<p>Do not forget this.</p>"));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn renders_note_with_type() {
        let html = render_html(&parse("::note{type=warning}\nWatch out.\n::").document);
        assert!(html.contains("class=\"note note--warning\""));
        assert!(html.contains("role=\"note\""));
    }

    #[test]
    fn renders_note_with_unknown_type_as_plain_note() {
        let html = render_html(&parse("::note{type=banana}\nHmm.\n::").document);
        assert!(html.contains("class=\"note\""));
        assert!(!html.contains("note--banana"));
    }

    #[test]
    fn renders_aside_directive() {
        let html = render_html(&parse("::aside\nSidebar content.\n::").document);
        assert!(html.contains("<aside class=\"aside\">"));
        assert!(html.contains("<p>Sidebar content.</p>"));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn renders_figure_directive() {
        let html = render_html(&parse("::figure[A fine photo]\n![photo](/img.jpg)\n::").document);
        assert!(html.contains("<figure>"));
        assert!(html.contains("<img src=\"/img.jpg\""));
        assert!(html.contains("<figcaption>A fine photo</figcaption>"));
        assert!(html.contains("</figure>"));
    }

    #[test]
    fn renders_figure_without_caption() {
        let html = render_html(&parse("::figure\n![photo](/img.jpg)\n::").document);
        assert!(html.contains("<figure>"));
        assert!(!html.contains("<figcaption>"));
    }

    #[test]
    fn renders_details_directive() {
        let html = render_html(
            &parse("::details[How does this work?]\nIt works like magic.\n::").document,
        );
        assert!(html.contains("<details class=\"details\">"));
        assert!(html.contains("<summary>How does this work?</summary>"));
        assert!(html.contains("<p>It works like magic.</p>"));
        assert!(html.contains("</details>"));
    }

    #[test]
    fn renders_details_without_summary() {
        let html = render_html(&parse("::details\nHidden content.\n::").document);
        assert!(html.contains("<details class=\"details\">"));
        assert!(html.contains("<summary>Details</summary>"));
    }

    #[test]
    fn renders_inline_spoiler() {
        let html = render_html(&parse("This is ||secret text|| here.").document);
        assert!(html.contains("<label class=\"spoiler\">"));
        assert!(html.contains("class=\"spoiler-toggle\""));
        assert!(html.contains("<span class=\"spoiler-content\">secret text</span>"));
    }

    #[test]
    fn renders_footnote_definition_with_reference_label() {
        let html = render_html(&parse("Body[^type]\n\n[^type]: Footnote body.").document);
        assert!(html.contains("<sup><a id=\"fnref-type\" href=\"#fn-type\">type</a></sup>"));
        assert!(html.contains(
            "<div id=\"fn-type\" class=\"footnote\" data-footnote-label=\"type\"><p class=\"footnote-label\"><sup><a href=\"#fnref-type\">type</a></sup></p>"
        ));
    }

    #[test]
    fn renders_task_lists_with_inline_checkbox_layout_wrapper() {
        let html = render_html(&parse("- [x] Done.\n- [ ] Next.").document);
        assert!(html.contains("<ul class=\"task-list\">"));
        assert!(html.contains("class=\"task-list-item__body\""));
        assert!(html.contains("class=\"task-list-item__content\"><p>Done.</p>"));
    }

    #[test]
    fn renders_heading_with_auto_id() {
        let html = render_html(&parse("# Hello World").document);
        assert!(html.contains("<h1 id=\"hello-world\">Hello World</h1>"));
    }

    #[test]
    fn renders_heading_preserves_explicit_id() {
        let html = render_html(&parse("# Hello {#custom}").document);
        assert!(html.contains("id=\"custom\""));
        assert!(!html.contains("id=\"hello\""));
    }

    #[test]
    fn renders_toc_directive() {
        let html = render_html(&parse("# One\n\n::toc\n::\n\n## Two\n\n## Three").document);
        assert!(html.contains("<nav class=\"toc\""));
        assert!(html.contains("<li><a href=\"#one\">One</a></li>"));
        assert!(html.contains("<li><a href=\"#two\">Two</a></li>"));
        assert!(html.contains("<li><a href=\"#three\">Three</a></li>"));
    }

    #[test]
    fn renders_pagebreak_directive() {
        let html = render_html(&parse("Before\n\n::pagebreak\n::\n\nAfter").document);
        assert!(html.contains("<div class=\"page-break\"></div>"));
    }

    #[test]
    fn renders_abbr_directive() {
        let html = render_html(
            &parse("The :abbr[HTML]{title=\"HyperText Markup Language\"} spec.").document,
        );
        assert!(html.contains("<abbr title=\"HyperText Markup Language\">HTML</abbr>"));
    }

    #[test]
    fn renders_abbr_without_title() {
        let html = render_html(&parse("The :abbr[CSS] spec.").document);
        assert!(html.contains("<abbr>CSS</abbr>"));
    }

    #[test]
    fn renders_kbd_directive() {
        let html = render_html(&parse("Press :kbd[Ctrl+Shift+P] to open.").document);
        assert!(html.contains("<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd>"));
    }

    #[test]
    fn renders_kbd_single_key() {
        let html = render_html(&parse("Press :kbd[Enter] to confirm.").document);
        assert!(html.contains("<kbd>Enter</kbd>"));
    }

    #[test]
    fn renders_cite_directive() {
        let html = render_html(&parse("As noted in :cite[The Art of War].").document);
        assert!(html.contains("<cite>The Art of War</cite>"));
    }

    #[test]
    fn renders_section_container() {
        let html = render_html(&parse(":::section{title=\"Intro\"}\nContent.\n:::").document);
        assert!(html.contains("<section"));
        assert!(html.contains("aria-label=\"Intro\""));
        assert!(html.contains("<p>Content.</p>"));
    }

    #[test]
    fn renders_chapter_container() {
        let html =
            render_html(&parse(":::chapter{title=\"One\"}\nChapter text.\n:::/chapter").document);
        assert!(html.contains("<section class=\"chapter\""));
        assert!(html.contains("aria-label=\"One\""));
    }

    #[test]
    fn renders_columns_layout() {
        let input =
            ":::columns{count=2}\n:::column\nLeft.\n:::\n:::column\nRight.\n:::\n:::/columns";
        let html = render_html(&parse(input).document);
        assert!(html.contains("<div class=\"columns\""));
        assert!(html.contains("--columns-count: 2"));
        assert!(html.contains("<div class=\"column\""));
    }

    #[test]
    fn renders_columns_with_gap() {
        let input = ":::columns{count=3 gap=\"2rem\"}\nContent.\n:::";
        let html = render_html(&parse(input).document);
        assert!(html.contains("--columns-count: 3"));
        assert!(html.contains("--columns-gap: 2rem"));
    }

    #[test]
    fn renders_standalone_pagebreak_and_toc_as_block_elements() {
        let html = render_html(&parse(":pagebreak\n\n:toc").document);

        assert!(html.contains("<div class=\"page-break\"></div>"));
        assert!(
            html.contains("<nav class=\"toc\" aria-label=\"Table of contents\"><ol></ol></nav>")
        );
    }

    #[test]
    fn renders_inline_math_as_mathml() {
        let html = render_html(&parse("The equation :math[\\frac{1}{2}] is simple.").document);
        assert!(html.contains(
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mfrac><mn>1</mn><mn>2</mn></mfrac></math>"
        ));
        assert!(!html.contains("data-etch-directive=\"math\""));
    }

    #[test]
    fn renders_display_math_as_mathml() {
        let html = render_html(&parse("::math\n\\int_0^1 x^2 \\, dx\n::").document);
        assert!(
            html.contains("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">")
        );
    }

    #[test]
    fn renders_container_math_as_mathml() {
        let html = render_html(&parse(":::math\n\\alpha + \\beta\n:::").document);
        assert!(
            html.contains("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">")
        );
        assert!(html.contains("<mi>α</mi>"));
        assert!(!html.contains("data-etch-directive=\"math\""));
    }

    #[test]
    fn renders_unknown_inline_directive_as_placeholder() {
        let html = render_html(&parse(":custom[hello]{foo=bar}").document);
        assert!(html.contains("data-etch-directive=\"custom\""));
        assert!(html.contains("data-etch-kind=\"inline\""));
        assert!(html.contains("data-etch-content=\"hello\""));
        assert!(html.contains("data-etch-attrs=\"{&quot;foo&quot;:&quot;bar&quot;}\""));
    }
}
