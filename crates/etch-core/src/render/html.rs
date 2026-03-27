use crate::{
    Alignment, Attributes, Block, DefinitionItem, Document, Frontmatter, Inline, ListItem,
    TableCell,
};
use std::collections::HashSet;

pub fn render_html(document: &Document) -> String {
    HtmlRenderer::default().render(document)
}

pub fn render_html_document(document: &Document) -> String {
    HtmlRenderer::default().render_document(document)
}

#[derive(Debug, Clone, Default)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, document: &Document) -> String {
        self.render_blocks(&document.body)
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
                wrap_with_tag("p", attrs.as_ref(), &[], &[], &self.render_inlines(content))
            }
            Block::Heading {
                level,
                content,
                attrs,
            } => {
                let tag = format!("h{}", (*level).clamp(1, 6));
                wrap_with_tag(
                    &tag,
                    attrs.as_ref(),
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
                let inner = items
                    .iter()
                    .map(|item| self.render_list_item(item))
                    .collect::<Vec<_>>()
                    .join("");

                wrap_with_tag(tag, attrs.as_ref(), &[], &[], &inner)
            }
            Block::Table {
                headers,
                rows,
                alignments,
                attrs,
            } => self.render_table(headers, rows, alignments, attrs.as_ref()),
            Block::ThematicBreak => "<hr>".to_string(),
            Block::BlockDirective {
                name,
                label,
                attrs,
                body,
            } => self.render_directive("div", name, label.as_deref(), attrs.as_ref(), body),
            Block::ContainerDirective {
                name,
                label,
                attrs,
                body,
                ..
            } => self.render_directive("section", name, label.as_deref(), attrs.as_ref(), body),
            Block::FootnoteDefinition { label, content } => wrap_with_tag(
                "div",
                None,
                &[
                    ("id", footnote_id(label)),
                    ("class", "footnote".to_string()),
                ],
                &[],
                &self.render_blocks(content),
            ),
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

        if let Some(checked) = item.checked {
            extra_attrs.push(("data-task", "true".to_string()));
            extra_attrs.push(("data-checked", checked.to_string()));
            inner.push_str("<input type=\"checkbox\" disabled");

            if checked {
                inner.push_str(" checked");
            }

            inner.push('>');
        }

        inner.push_str(&self.render_blocks(&item.content));

        wrap_with_tag("li", None, &extra_attrs, &[], &inner)
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

    fn render_directive(
        &self,
        tag: &str,
        name: &str,
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

        wrap_with_tag(
            tag,
            attrs,
            &[("data-directive", name.to_string())],
            &[],
            &inner,
        )
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
                    name,
                    content,
                    attrs,
                } => {
                    let rendered_content = content
                        .as_deref()
                        .map(|content| self.render_inlines(content))
                        .unwrap_or_default();

                    html.push_str(&wrap_with_tag(
                        "span",
                        attrs.as_ref(),
                        &[("data-directive", name.to_string())],
                        &[],
                        &rendered_content,
                    ));
                }
                Inline::FootnoteReference { label } => {
                    let reference = wrap_with_tag(
                        "a",
                        None,
                        &[("href", format!("#{}", footnote_id(label)))],
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

fn document_title(frontmatter: Option<&Frontmatter>) -> Option<String> {
    let frontmatter = frontmatter?;
    let value = frontmatter.fields.get("title")?;

    value.as_title_string()
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
                    name: "aside".to_string(),
                    label: Some(vec![Inline::Text {
                        value: "Note".to_string(),
                    }]),
                    attrs: None,
                    body: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            value: "Directive body".to_string(),
                        }],
                        attrs: None,
                    }],
                },
                Block::ContainerDirective {
                    name: "chapter".to_string(),
                    label: None,
                    attrs: None,
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
                "<p>Hello <em>world</em> and <strong>friends</strong>.</p>\n",
                "<pre><code class=\"language-rust\">fn main() {}</code></pre>\n",
                "<ul><li data-task=\"true\" data-checked=\"true\"><input type=\"checkbox\" disabled checked><p>Todo</p></li></ul>\n",
                "<table><thead><tr><th style=\"text-align: center;\">Name</th></tr></thead><tbody><tr><td style=\"text-align: center;\"><a href=\"https://example.com\" title=\"Go\">Etch</a></td></tr></tbody></table>\n",
                "<div data-directive=\"aside\"><p class=\"directive-label\">Note</p>\n<p>Directive body</p></div>\n",
                "<section data-directive=\"chapter\"><p><sup><a href=\"#fn-a\">a</a></sup></p></section>\n",
                "<div id=\"fn-a\" class=\"footnote\"><p>Footnote</p></div>"
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

        assert!(html.contains("<h1>Embers in the Snow</h1>"));
        assert!(html.contains("<div data-directive=\"dedication\"><p>For everyone who stayed up too late reading under the covers.</p></div>"));
        assert!(html.contains(
            "<section data-directive=\"chapter\" data-number=\"1\" data-title=\"The First Snow\">"
        ));
        assert!(html.contains("<mark>absolute</mark>"));
        assert!(html.contains("<blockquote><p><em>\"I'll come back when the embers remember how to burn.\"</em></p>\n<p class=\"attribution\">"));
        assert!(
            html.contains("<span data-directive=\"math\">3\\text{m} \\times 3\\text{m}</span>")
        );
        assert!(
            html.contains(
                "Her address was simple:<br>42 Northwind Road<br>The Village at the Edge"
            )
        );
        assert!(html.contains("<sup><a href=\"#fn-1\">1</a></sup>"));
        assert!(html.contains("<div id=\"fn-1\" class=\"footnote\"><p>The finding is loosely based on a Scandinavian\nfolktale about returning wolves.</p></div>"));
    }
}
