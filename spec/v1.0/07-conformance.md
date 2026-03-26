# 9. Conformance

This chapter defines parser-wide conformance requirements that apply across the rest of the Etch specification. It consolidates the block-recognition order, the escape rules that have language-wide effect, the required distinction between errors and warnings, the full list of CommonMark divergences, and one fully annotated integration example.

The approved integration corpus in `tests/corpus/integration/` is informative for this chapter. In particular, `minimal.etch`, `plain-story.etch`, `technical-doc.etch`, `everything.etch`, and `embers-in-the-snow.etch` demonstrate the intended interaction of the constructs defined in Chapters 4 through 8.

## 9.1 Parsing Priority

Block recognition is ordered. After frontmatter handling and comment stripping, a conforming parser MUST test each candidate block start against the following priority order and MUST stop at the first matching construct.

### Structure

1. **Frontmatter**: Frontmatter recognition occurs before all other block recognition and only when a line containing exactly `---` appears on the very first line of the file, as defined in Section 8.2.
2. **Comments**: After frontmatter is handled, the parser MUST behave as though comments had been removed from the remaining source before block parsing and inline parsing begin. This requirement applies everywhere except inside fenced code block bodies, where comment syntax is inactive. An implementation MAY realize this through a literal preprocessing pass or by any equivalent strategy that produces the same result.
3. **Fenced code blocks**: At a block start, a line beginning with `` ``` `` MUST be recognized as a fenced code block before any directive, heading, blockquote, thematic break, list, table, footnote-definition, definition-list, or paragraph rule is attempted.
4. **Container directives**: A line beginning with `:::` immediately followed by a valid directive name MUST be recognized as a container directive before block directives and all later block constructs.
5. **Block directives**: A line beginning with `::` immediately followed by a valid directive name MUST be recognized as a block directive before headings and all later block constructs.
6. **Headings**: A valid ATX heading line MUST be recognized before blockquotes and all later block constructs.
7. **Blockquotes**: A line whose first character is `>` MUST be recognized as a blockquote before thematic breaks and all later block constructs.
8. **Thematic breaks**: A valid thematic break line MUST be recognized before list parsing and all later block constructs.
9. **Lists**: A valid list-item opener MUST be recognized before tables, footnote definitions, definition lists, and paragraphs.
10. **Tables**: A valid pipe-table header line followed immediately by a valid separator line MUST be recognized before footnote definitions, definition lists, and paragraphs.
11. **Footnote definitions**: A valid footnote-definition opener MUST be recognized before definition lists and paragraphs.
12. **Definition lists**: A valid term line followed immediately by a valid `: ` definition opener MUST be recognized before paragraph fallback.
13. **Paragraphs**: A paragraph is the final fallback. Any non-blank line not consumed by a higher-priority rule begins or continues a paragraph.
14. **Recursive application**: The same recognition order applies recursively inside nested block contexts such as blockquotes, list items, footnote definitions, block directives, and container directives, subject to the nesting constraints defined in Chapter 7 and Section 9.3.

### Normative Examples

`✓ Code fences outrank directive-looking text`

````etch
```etch
:::chapter
::aside
{~ hidden? ~}
```
````

This input produces one `CodeBlock`. The apparent container directive, block directive, and comment markers remain literal code text because fenced code blocks are recognized before those constructs and suspend their parsing inside the body.

`✓ Paragraph is the final fallback`

```etch
note: this stays a paragraph line
: not a directive either
```

This input produces one `Paragraph`. Neither line begins a higher-priority block construct, so paragraph fallback applies.

## 9.2 Escaping Rules

Etch uses backslash escaping in a small number of specific contexts. A conforming parser MUST apply only the escape rules defined in this section and in the construct-specific chapters they summarize. Outside these cases, a backslash is literal source text.

### Structure

1. **Hard line breaks**: In inline-bearing block content, a trailing backslash immediately before a line ending produces a `HardBreak` inline node. This is the only Etch hard-line-break syntax.
2. **Inline delimiter escapes**: During ordinary inline parsing, a backslash escapes the next character only when that character is one of `*`, `~`, `^`, `=`, `+`, `[`, `]`, or `\`. The backslash is consumed and the following character is emitted as literal text.
3. **Effect on delimiter runs**: Because `=`, `+`, `*`, `~`, and `^` are escapable, sequences such as `\*`, `\~`, `\^`, `\=\=`, and `\+\+` suppress the corresponding inline constructs and produce literal punctuation instead.
4. **Directive bracketed content and labels**: Bracketed directive content uses balanced-bracket counting first. When a literal unmatched bracket is needed inside the bracketed region, `\]` and `\[` MAY be used so that the bracket does not affect the balancing scan. After the bracketed region is delimited, ordinary inline escape rules apply to its content.
5. **Quoted attribute values**: Inside a quoted attribute value, `\"` escapes a literal double quote. This is the only Etch-defined escape inside attribute values. Other backslashes remain part of the attribute value.
6. **Code contexts**: Inline code spans and fenced code block bodies do not process Etch escapes. Backslashes inside those contexts remain literal code text.
7. **Comment contexts**: Comments do not process Etch escapes. Backslashes inside `{~ ... ~}` are part of the comment source and disappear only because the entire comment is removed from the document model.
8. **Frontmatter**: Frontmatter content is YAML, not Etch inline content. Any escaping inside frontmatter is governed by YAML rather than by Etch.
9. **No dollar-sign escape rule**: Because Etch does not define `$...$` math syntax, this specification defines no special backslash escape for `$`.
10. **Literal backslashes elsewhere**: A backslash that does not participate in one of the rules above MUST remain literal text.

### Normative Examples

`✓ Escaped inline punctuation`

```etch
\*not italic\* stays literal at the start.
This sentence keeps \~not sub\~, \^not super\^, and \=\=not highlight\=\= literal.
This sentence ends with \+\+not insert\+\+.
```

This input, from `tests/corpus/core/inline/escaped.etch`, produces literal punctuation rather than inline formatting nodes.

`✓ Escaped brackets inside directive content`

```etch
The tooltip says :tooltip[use \] to close early] in the parser guide.
The note reads :note[this has one \[ extra bracket].
```

This input, from `tests/corpus/extensions/inline-directives/escaped-bracket.etch`, demonstrates `\]` and `\[` inside balanced directive content.

`✓ Escaped quote inside an attribute value`

```etch
Quoted label {key="value with \"escaped quotes\" inside"}
```

This input, from `tests/corpus/extensions/attributes/escaped-quote.etch`, demonstrates the required escape form for a literal `"` inside a quoted attribute value.

`✓ No escape processing inside inline code`

```etch
`\*not italic\* and \[literal\] \\`
```

This input produces one `InlineCode` node whose literal value still contains the backslashes.

## 9.3 Error Handling

Etch distinguishes hard parse errors from soft parse warnings. A conforming parser MUST report the issue classes defined here and SHOULD continue parsing when it can still recover a useful partial document model.

### Structure

1. **Issue classes**: Parse issues are either `Error` or `Warning`.
2. **Mismatched named close**: When a container directive opened as `:::name` is later closed by `:::/other-name`, the parser MUST report an `Error`.
3. **Directive inside a leaf directive**: When a block directive or container directive appears inside the body of a leaf directive, the parser MUST report an `Error`.
4. **Unclosed directive**: When a block directive or container directive reaches end of input without a valid closing fence, the parser MUST report an `Error`.
5. **Structural nesting warning threshold**: Structural directive nesting that reaches four levels or deeper SHOULD produce a `Warning`. This is not a parse error and does not make the document invalid.
6. **Partial parsing requirement**: Even when one or more `Error` conditions occur, a conforming parser SHOULD continue parsing as far as practical and SHOULD return a partial AST together with the accumulated issue list.
7. **Recovery after mismatched named close**: When a mismatched named close is encountered, the parser SHOULD close the current container at that location, record the error, and preserve the container body parsed up to that point.
8. **Recovery after directive-in-leaf error**: When a nested directive is encountered inside a leaf directive, the parser SHOULD preserve the nested directive in the partial AST while also recording the error.
9. **Recovery after unclosed directive**: When a directive is unclosed at end of input, the parser SHOULD preserve the directive node and the body parsed up to end of input while also recording the error.
10. **Issue location**: Error and warning reports SHOULD include at least the source line where the issue was detected. A column number SHOULD be included when the implementation can determine it.

### Normative Examples

`✗ Mismatched named close`

```etch
:::chapter{title="One"}
This container is closed with the wrong directive name.
:::/section
```

This input, from `tests/corpus/extensions/container-directives/mismatched-close.etch`, is invalid. A conforming parser MUST report an `Error`. The current snapshot preserves the `chapter` container and its body while recording the close mismatch.

`✗ Directive nested inside a leaf directive`

```etch
::aside
This leaf directive starts with valid text.

::spoiler
This nested directive should cause a parse error.
::
::
```

This input, from `tests/corpus/extensions/nesting/leaf-rejects-directive.etch`, is invalid. A conforming parser MUST report an `Error`. A parser SHOULD still preserve both directive nodes in the partial AST.

`✗ Unclosed directive`

```etch
::aside
This body reaches end of input without a closing fence.
```

This input is invalid. A conforming parser MUST report an `Error` for the missing closing fence and SHOULD still preserve the `BlockDirective` and its parsed body in the partial AST.

`✓ Valid but warns at structural depth four`

```etch
:::chapter{title="One"}
:::section{title="Two"}
:::columns{count=2}
:::column
Deep content lives here.
:::/column
:::/columns
:::/section
:::/chapter
```

This input, from `tests/corpus/extensions/nesting/depth-4-warning.etch`, is valid. A conforming parser SHOULD report a `Warning` when the fourth structural container is opened.

## 9.4 CommonMark Divergences

This section consolidates the full set of normative Etch divergences from CommonMark already introduced throughout this specification.

### Structure

1. **Hard line breaks**: Etch recognizes a hard line break only from a trailing backslash before the line ending. Two trailing spaces MUST NOT create a hard line break.
2. **Reference links**: Etch does not define reference-style links. Only inline link syntax and bare `http://` / `https://` autolinks are defined.
3. **Setext headings**: Etch does not define setext headings. Only ATX headings using `#` through `######` are headings.
4. **Indented code blocks**: Etch does not define indented code blocks. Only fenced code blocks delimited by `` ``` `` are code blocks.
5. **Raw HTML**: Etch does not define raw HTML blocks or raw inline HTML as language elements. Etch is AST-first and does not treat HTML as a fallback syntax.
6. **Autolinks**: Etch automatically links bare `http://` and `https://` URLs without requiring angle brackets or link markup. Bare domains without a recognized scheme are not autolinks.
7. **Inline additions**: Etch adds strikethrough (`~~...~~`), superscript (`^...^`), subscript (`~...~`), highlight (`==...==`), and inserted text (`++...++`).
8. **List additions and indentation**: Etch adds task list markers (`- [ ]` and `- [x]`) and defines list nesting by a threshold of two or more spaces rather than by CommonMark's indentation rules. Two spaces are the canonical serialized form for one nesting level.
9. **Table support**: Etch adds pipe tables.
10. **Footnotes**: Etch adds footnote references and footnote definitions.
11. **Definition lists**: Etch adds definition lists.
12. **Frontmatter**: Etch adds YAML frontmatter delimited by `---` on the first line of the file only.
13. **Comments**: Etch adds comment syntax using `{~ ... ~}`. Comments are stripped from the document model.
14. **Attributes**: Etch adds attribute sets using `{#id .class key=value}` and related forms.
15. **Directives**: Etch adds inline directives, block directives, and container directives. This includes Etch's directive disambiguation rule that `:` begins a directive only when immediately followed by a letter, its directive-name character restrictions, and its fenced block/container directive forms.
16. **Math syntax**: Etch defines math only through directives such as `:math[...]` and `::math ... ::`. Etch does not define `$...$` or `$$...$$` math syntax.

## 9.5 Full Document Example

The following example is the canonical integration document from `tests/corpus/integration/embers-in-the-snow.etch`. Its current AST is snapshotted in `crates/etch-core/tests/snapshots/corpus_tests__integration-embers-in-the-snow.snap`.

### Example

```etch
---
title: "Embers in the Snow"
author: "foxwriter42"
date: 2026-03-20
tags: [drama, winter, shortform]
rating: general
---

{~ This is the canonical example document for the Etch spec. ~}

# Embers in the Snow

::dedication
For everyone who stayed up too late reading under the covers.
::

::content-warning{level=mild tags="melancholy"}
Themes of nostalgia and gentle sadness.
::

:::chapter{number=1 title="The First Snow"}

The village woke to silence. Not the silence of sleep,
but the silence of snow — heavy, patient, ==absolute==.

::aside
The author grew up in a town like this one.
::

:character[Mira]{species=wolf} stepped onto the porch,
breath curling like smoke. She held a letter — creased
from folding and refolding — and read it one last time.

> *"I'll come back when the embers remember how to burn."*
>
> — Kael's last letter {.attribution}

She tucked it into her coat and walked into the white.

:::/chapter

---

:::chapter{number=2 title="The Finding"}

Three days later, she found what she was looking for.[^1]

Not Kael — not exactly. But the place where he'd been,
marked by a ring of stones and the ash of a fire
that might have been warm ==yesterday==.

The area measured roughly :math[3\text{m} \times 3\text{m}],
with the stones arranged in a precise circle.

Her address was simple:\
42 Northwind Road\
The Village at the Edge

:::/chapter

[^1]: The finding is loosely based on a Scandinavian
folktale about returning wolves.
```

### Annotation

1. Lines 1 through 7 are frontmatter. Because the opening `---` is on the first line, the block is parsed as frontmatter rather than as a thematic break.
2. Line 9 is a comment. It is removed before the document body is parsed and therefore contributes no AST node.
3. Line 11 is an ATX heading.
4. Lines 13 through 15 form a block directive named `dedication`.
5. Lines 17 through 19 form a block directive named `content-warning` with attributes.
6. Lines 21 through 40 form a structural container directive named `chapter`, closed with a named close.
7. The paragraph beginning on line 23 demonstrates ordinary paragraph continuation across physical lines and produces `SoftBreak` nodes.
8. The span `==absolute==` demonstrates highlight syntax inside a paragraph.
9. Lines 26 through 28 show a leaf block directive nested inside a structural container, which is valid.
10. The span `:character[Mira]{species=wolf}` demonstrates an inline directive with both bracketed content and attributes.
11. Lines 34 through 36 form a blockquote containing an emphasized paragraph and a second paragraph with paragraph-level attributes.
12. Line 42 is a thematic break because document content has already begun; at this position `---` cannot be frontmatter.
13. The second `chapter` container shows that multiple top-level containers may be separated by ordinary block syntax.
14. The span `[^1]` is a footnote reference, and lines 61 through 62 provide the matching footnote definition.
15. The span `:math[3\text{m} \times 3\text{m}]` demonstrates math expressed through an inline directive rather than through dollar-sign syntax.
16. The three-line address demonstrates Etch hard line breaks: each trailing backslash becomes a `HardBreak` node.
17. The document as a whole demonstrates the intended interoperability of frontmatter, comments, headings, block directives, container directives, inline directives, highlights, blockquotes, attributes, thematic breaks, footnotes, inline math, soft breaks, and hard breaks within one conforming Etch source file.
