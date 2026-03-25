// crates/etch-core/src/ast.rs
//
// The complete AST for the Etch markup language (v0.3 spec).
// This file defines every node type the parser can produce.
// All types derive Serialize so they can be snapshot-tested as JSON.

use serde::Serialize;
use std::collections::HashMap;

// ─────────────────────────────────────────────
// Top-level document
// ─────────────────────────────────────────────

/// A complete Etch document. This is what parse() returns.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Document {
    /// YAML frontmatter (None if the file doesn't start with ---)
    pub frontmatter: Option<Frontmatter>,
    /// The document body: a sequence of block-level elements
    pub body: Vec<Block>,
}

// ─────────────────────────────────────────────
// Frontmatter
// ─────────────────────────────────────────────

/// YAML frontmatter parsed from the --- ... --- block at the start of a file.
/// Only present if the very first line of the file is ---.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frontmatter {
    /// The raw YAML text (useful for round-tripping)
    pub raw: String,
    /// Parsed key-value pairs. Values can be strings, numbers, arrays, or nested objects.
    /// We use serde_yaml::Value to represent arbitrary YAML.
    pub fields: HashMap<String, serde_yaml::Value>,
}

// ─────────────────────────────────────────────
// Block-level elements
// ─────────────────────────────────────────────

/// A block-level element in the document.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Block {
    /// A paragraph: one or more lines of inline content separated by blank lines.
    Paragraph {
        content: Vec<Inline>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A heading (# through ######). Level is 1-6.
    Heading {
        level: u8,
        content: Vec<Inline>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A fenced code block (``` ... ```).
    /// Content is raw text — no inline parsing inside.
    CodeBlock {
        /// Language identifier (e.g., "rust", "python"). None if not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// The raw text content of the code block.
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A blockquote (> lines). Contains recursively parsed blocks.
    BlockQuote {
        content: Vec<Block>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// An ordered or unordered list.
    List {
        /// true = ordered (1. 2. 3.), false = unordered (- - -)
        ordered: bool,
        /// The list items.
        items: Vec<ListItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A pipe-delimited table.
    Table {
        /// Header row cells, each containing inline content.
        headers: Vec<TableCell>,
        /// Data rows, each a vector of cells.
        rows: Vec<Vec<TableCell>>,
        /// Column alignments.
        alignments: Vec<Alignment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A thematic break (---, ***, or ___).
    ThematicBreak,

    /// A block directive (::name ... ::). Leaf by default.
    /// Can contain rich content including multiple paragraphs (fenced behavior).
    /// Cannot contain other directives (unless overridden by plugin to structural).
    BlockDirective {
        /// The directive name (e.g., "aside", "math", "spoiler").
        /// Letters and hyphens only, starts with a letter.
        name: String,
        /// Optional label text in [brackets] after the name.
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<Vec<Inline>>,
        /// Optional attributes in {braces} after the name/label.
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
        /// The body content, parsed as blocks.
        body: Vec<Block>,
    },

    /// A container directive (:::name ... ::: or :::/name). Structural by default.
    /// Can contain other directives (block and container).
    ContainerDirective {
        /// The directive name (e.g., "chapter", "columns", "column").
        name: String,
        /// Optional label text in [brackets].
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<Vec<Inline>>,
        /// Optional attributes in {braces}.
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
        /// The body content, parsed as blocks (may include nested directives).
        body: Vec<Block>,
        /// Whether the directive was closed with a named close (:::/name).
        named_close: bool,
    },

    /// A footnote definition: [^label]: content
    FootnoteDefinition {
        /// The footnote label (e.g., "1", "my-note").
        label: String,
        /// The footnote content, parsed as blocks.
        content: Vec<Block>,
    },

    /// A definition list: Term / : Definition pairs.
    DefinitionList {
        items: Vec<DefinitionItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
}

// ─────────────────────────────────────────────
// Inline-level elements
// ─────────────────────────────────────────────

/// An inline-level element within a paragraph, heading, list item, etc.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Inline {
    /// Plain text with no formatting.
    Text { value: String },

    /// *italic* — emphasis.
    Emphasis { content: Vec<Inline> },

    /// **bold** — strong emphasis.
    Strong { content: Vec<Inline> },

    /// ~~struck~~ — strikethrough.
    Strikethrough { content: Vec<Inline> },

    /// `code` — inline code. Content is raw text (no nested formatting).
    InlineCode { value: String },

    /// ^super^ — superscript.
    Superscript { content: Vec<Inline> },

    /// ~sub~ — subscript.
    Subscript { content: Vec<Inline> },

    /// ==marked== — highlighted text.
    Highlight { content: Vec<Inline> },

    /// ++inserted++ — inserted text.
    Insert { content: Vec<Inline> },

    /// [text](url "title") — a hyperlink.
    Link {
        /// The URL target.
        url: String,
        /// Optional title text.
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// The link text, which can contain inline formatting.
        content: Vec<Inline>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// ![alt](url "title") — an image.
    Image {
        /// The image URL/path.
        url: String,
        /// Alt text for accessibility.
        alt: String,
        /// Optional title/caption.
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// A bare URL that was auto-linked (http:// or https:// only).
    AutoLink { url: String },

    /// :name[content]{attrs} — an inline directive.
    InlineDirective {
        /// Directive name (letters and hyphens, starts with letter).
        name: String,
        /// Optional content in [brackets]. Can contain inline formatting.
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<Inline>>,
        /// Optional attributes in {braces}.
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },

    /// [^label] — a reference to a footnote.
    FootnoteReference { label: String },

    /// A soft line break (newline within a paragraph, typically rendered as a space).
    SoftBreak,

    /// A hard line break (trailing \ at end of line, rendered as <br>).
    HardBreak,
}

// ─────────────────────────────────────────────
// Supporting types
// ─────────────────────────────────────────────

/// Attributes that can be attached to any element: {#id .class key=value}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Attributes {
    /// The element ID (#id).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// CSS classes (.class1 .class2).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<String>,
    /// Key-value pairs (key=value, key="quoted value").
    /// Values with escaped quotes are stored unescaped.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub pairs: HashMap<String, String>,
}

/// A table cell containing inline content.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

/// Column alignment in a table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Alignment {
    /// No alignment specified (default, typically left).
    None,
    /// Left-aligned (:---).
    Left,
    /// Center-aligned (:---:).
    Center,
    /// Right-aligned (---:).
    Right,
}

/// A single item in a list.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListItem {
    /// The item content, parsed as blocks (can contain paragraphs, nested lists, etc.)
    pub content: Vec<Block>,
    /// For task lists: Some(true) = checked [x], Some(false) = unchecked [ ].
    /// None for regular (non-task) list items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
}

/// A term-definition pair in a definition list.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DefinitionItem {
    /// The term being defined (can contain inline formatting).
    pub term: Vec<Inline>,
    /// One or more definitions for this term.
    /// Each definition is a sequence of blocks.
    pub definitions: Vec<Vec<Block>>,
}

// ─────────────────────────────────────────────
// Parse errors and warnings
// ─────────────────────────────────────────────

/// An error or warning produced during parsing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParseError {
    /// The kind of issue.
    pub kind: ParseErrorKind,
    /// Human-readable description.
    pub message: String,
    /// Line number (1-indexed) where the issue was detected.
    pub line: usize,
    /// Optional column number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

/// Whether a parse issue is a hard error or a soft warning.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ParseErrorKind {
    /// A hard error: the document is malformed.
    /// Examples: mismatched named close, directive inside leaf, unclosed directive.
    Error,
    /// A soft warning: the document is valid but may have issues.
    /// Examples: structural nesting beyond 4 levels.
    Warning,
}

// ─────────────────────────────────────────────
// Parse result
// ─────────────────────────────────────────────

/// The result of parsing an Etch document.
/// Contains both the AST and any errors/warnings encountered.
/// Even if errors are present, a partial AST may still be returned
/// (for editor integration where partial results are useful).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParseResult {
    /// The parsed document (may be partial if errors occurred).
    pub document: Document,
    /// Any errors or warnings encountered during parsing.
    pub errors: Vec<ParseError>,
}
