# 1. Introduction

Etch is a plain-text markup language for writing structured documents. It preserves the direct readability of Markdown-style source text while extending the language through a single directive system that applies uniformly to inline, block, and container constructs. An Etch document defines document structure independently of any particular renderer, editor, or output format.

## 1.1 Design Principles

The Etch language is defined by the following principles:

- Plain text is the source of truth. A conforming Etch document MUST remain readable as plain text without requiring rendered output.
- Complexity is opt-in. An empty file is a valid Etch document, and authors MAY use only the subset of constructs required by a given document.
- Extension syntax is uniform. Features beyond the core Markdown-compatible surface MUST use the directive forms defined by this specification rather than introducing feature-specific extension syntaxes.
- The core language is platform-agnostic. This specification MUST define the meaning of core syntax independently of any platform-specific directive vocabulary or rendering environment.
- Structure is normative; rendering is derived. This specification defines the document structure produced by Etch source and does not require any particular visual presentation.

## 1.2 Relationship to CommonMark

Etch is specified as a CommonMark superset with explicit divergences. Unless this specification states otherwise, CommonMark constructs retain their usual meaning in Etch. Where this specification defines a divergence, Etch behavior MUST follow this specification.

Etch diverges from CommonMark in the following ways:

- Etch adds strikethrough using `~~...~~`.
- Etch adds superscript using `^...^`.
- Etch adds subscript using `~...~`.
- Etch adds highlight using `==...==`.
- Etch adds inserted text using `++...++`.
- Etch adds task list markers within list items using `- [ ]` and `- [x]`.
- Etch adds pipe tables.
- Etch adds footnote references and footnote definitions.
- Etch adds definition lists.
- Etch adds document frontmatter delimited by `---` on the first line of the file only.
- Etch adds comment syntax using `{~ ... ~}`. Comments are not part of the document model.
- Etch adds attributes using `{#id .class key=value}`. Attributes MAY be attached to block and inline elements.
- Etch adds inline directives using `:name`, `:name[...]`, `:name{...}`, and `:name[...]{...}`.
- In Etch, `:` begins a directive only when it is immediately followed by a letter.
- In Etch, directive names consist only of letters and hyphens and MUST begin with a letter.
- Etch adds fenced block directives using `::name ... ::`.
- In Etch, blank lines do not terminate a block directive; only its closing fence terminates it.
- Etch adds container directives using `:::name ... :::` and `:::name ... :::/name`.
- Etch automatically links bare `http://` and `https://` URLs.
- Etch defines a hard line break only for a trailing backslash at the end of a line. Two trailing spaces MUST NOT create a hard line break.
- Etch treats an indentation of two or more spaces as one list nesting level deeper and defines two spaces as the canonical serialized form for one nesting level.
- Etch does not define raw HTML blocks or raw inline HTML as language elements.

## 1.3 File Format

An Etch source document MUST be encoded as UTF-8 plain text. Files containing Etch source SHOULD use the `.etch` file extension.

# 2. Document Model

An Etch document consists of, in order:

1. an optional frontmatter block; and
2. a sequence of block elements.

Comments MAY appear in source text, but comments do not contribute block elements, inline elements, or text nodes to the document model.

Block elements contain either block elements, inline elements, or literal text, depending on their type. Inline elements contain either inline elements or literal text, depending on their type.

## 2.1 Document

A document contains at most one frontmatter block. If present, frontmatter precedes all block elements. The remainder of the document is an ordered sequence of block elements.

## 2.2 Block Taxonomy

The block element types are:

- `Paragraph`: a run of inline content.
- `Heading`: a heading with a level from 1 through 6 and inline content.
- `CodeBlock`: a fenced code block containing literal text and an optional language identifier.
- `BlockQuote`: a quoted region containing a sequence of block elements.
- `List`: an ordered or unordered list containing list items. Each list item contains a sequence of block elements and MAY carry a task state.
- `Table`: a table containing header cells, data cells, and column alignment. Table cells contain inline content.
- `ThematicBreak`: a block separator with no child content.
- `BlockDirective`: a fenced leaf directive with a directive name, an optional inline label, optional attributes, and a body consisting of block elements.
- `ContainerDirective`: a structural directive with a directive name, an optional inline label, optional attributes, and a body consisting of block elements.
- `FootnoteDefinition`: a labeled footnote definition containing a sequence of block elements.
- `DefinitionList`: a sequence of definition-list items. Each item contains a term expressed as inline content and one or more definitions expressed as sequences of block elements.

## 2.3 Inline Taxonomy

The inline element types are:

- `Text`: literal text.
- `Emphasis`: emphasized inline content.
- `Strong`: strongly emphasized inline content.
- `Strikethrough`: struck inline content.
- `InlineCode`: literal inline code text.
- `Superscript`: superscript inline content.
- `Subscript`: subscript inline content.
- `Highlight`: highlighted inline content.
- `Insert`: inserted inline content.
- `Link`: a hyperlink with a destination, optional title, inline link text, and optional attributes.
- `Image`: an image with a source, alternative text, optional title, and optional attributes.
- `AutoLink`: a URL recognized as a link without explicit link markup.
- `InlineDirective`: an inline directive with a directive name, optional inline content, and optional attributes.
- `FootnoteReference`: a reference to a footnote definition by label.
- `SoftBreak`: a line break within a paragraph that does not force a hard line break.
- `HardBreak`: a forced line break within a paragraph.

## 2.4 Attributes

Attributes are an orthogonal attachment that MAY be associated with any block element or inline element. An attribute set consists of:

- at most one identifier;
- zero or more class names; and
- zero or more key/value pairs.

When an element carries attributes, those attributes modify that element only. Attributes do not implicitly apply to sibling elements, descendant elements, or ancestor elements.

# 3. Notation Conventions

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, and `OPTIONAL` in this specification are to be interpreted as described in RFC 2119.

Normative examples are identified by the markers `✓` and `✗`:

- `✓` marks an example that a conforming Etch implementation MUST accept as valid.
- `✗` marks an example that a conforming Etch implementation MUST reject or report as an error.

Unless an example states otherwise, the example input is shown in a code block and the required interpretation or error condition is described in the text immediately following the code block.
