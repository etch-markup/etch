# 5. Inline Syntax

This chapter defines the core inline constructs of Etch. Inline parsing occurs within paragraphs, headings, table cells, footnote definitions, directive labels, and other contexts that admit inline content.

Inline parsing is ordered. A conforming parser MUST scan inline content from left to right and MUST apply the following recognition order at each position:

1. hard line breaks introduced by a trailing backslash;
2. backslash escapes for literal inline punctuation;
3. soft line breaks produced by line endings;
4. autolinks beginning with `http://` or `https://`;
5. images beginning with `![`;
6. footnote references beginning with `[^`;
7. links beginning with `[`;
8. inline code spans beginning with one or two backticks; and
9. delimiter-based spans using `*`, `~`, `^`, `==`, and `++`.

For delimiter-based spans, the following general rules apply unless a subsection states otherwise:

1. An opening delimiter is valid only when the character immediately after the opening run exists and is not whitespace.
2. A closing delimiter is valid only when the character immediately before the closing run exists and is not whitespace.
3. A span with empty content MUST NOT be recognized and its delimiters remain literal text.
4. Unsupported delimiter-run lengths remain literal text. In particular, the inline parser recognizes only `*`, `**`, `***`, `~`, `~~`, `^`, `==`, `++`, `` ` ``, and `` `` `` as opening runs.

## 5.1 Emphasis

### Structure

1. **Opening delimiter**: A single asterisk: `*`.
2. **Closing delimiter**: A single asterisk: `*`.
3. **Flanking rules**: The opening `*` is valid only when followed by a non-whitespace character. The closing `*` is valid only when preceded by a non-whitespace character. `* *`, `* text*`, and `*text *` MUST remain literal text.
4. **Content model**: Emphasis content is parsed recursively as inline content. It MAY contain text, links, images, autolinks, code spans, footnote references, soft breaks, hard breaks, and other delimiter-based spans.
5. **Nesting**: Emphasis MAY contain other inline forms. A valid closing single `*` closes the current emphasis span before later parsing continues.
6. **Precedence**: At a run of asterisks, the parser classifies the run by length before parsing content. A single `*` is considered emphasis only when the run length is exactly one; `**` and `***` are not split and retried as single-asterisk emphasis openers.

### Normative Examples

`✓ Basic`

```etch
*Italic* opens this sentence.
This sentence has *italic* in the middle.
This sentence ends with *italic*.
```

This input, from `tests/corpus/core/inline/emphasis.etch`, produces `Emphasis` nodes at the start, middle, and end of inline content.

`✓ Edge case`

```etch
*Starts on this line
and ends on the next* in the same paragraph.
```

This input, from `tests/corpus/core/inline/across-lines.etch`, produces one `Emphasis` node whose content contains a `SoftBreak` between the two text nodes.

## 5.2 Strong

### Structure

1. **Opening delimiter**: Two consecutive asterisks: `**`.
2. **Closing delimiter**: Two consecutive asterisks: `**`.
3. **Flanking rules**: The opening `**` is valid only when followed by a non-whitespace character. The closing `**` is valid only when preceded by a non-whitespace character. `**`, `** **`, `** text**`, and `**text **` MUST remain literal text.
4. **Content model**: Strong content is parsed recursively as inline content.
5. **Nesting**: Strong MAY contain emphasis and the other inline forms defined by this chapter.
6. **Precedence**: A run of exactly two `*` characters is parsed as strong. A run of exactly three `*` characters is parsed as strong+emphasis, not as strong followed by a literal `*`.

### Normative Examples

`✓ Basic`

```etch
**Bold** opens this sentence.
This sentence has **bold** in the middle.
This sentence ends with **bold**.
```

This input, from `tests/corpus/core/inline/strong.etch`, produces `Strong` nodes in each position.

`✓ Edge case`

```etch
**Bold starts here
and ends here** in the same paragraph too.
```

This input, from `tests/corpus/core/inline/across-lines.etch`, produces one `Strong` node containing a `SoftBreak`.

## 5.3 Strong+Emphasis

### Structure

1. **Opening delimiter**: Three consecutive asterisks: `***`.
2. **Closing delimiter**: Three consecutive asterisks: `***`.
3. **Flanking rules**: The opening `***` is valid only when followed by a non-whitespace character. The closing `***` is valid only when preceded by a non-whitespace character. `*** ***`, `*** text***`, and `***text ***` MUST remain literal text.
4. **Content model**: Strong+emphasis content is parsed recursively as inline content.
5. **Nesting**: A recognized `***...***` span produces a `Strong` node whose sole child is an `Emphasis` node containing the parsed inner content.
6. **Precedence**: When the parser encounters a run of exactly three `*` characters, it MUST parse that run as strong+emphasis before considering any decomposition into `**` plus `*` or `*` plus `**`.

### Normative Examples

`✓ Basic`

```etch
***Bold italic*** opens this sentence.
This sentence has ***bold italic*** in the middle.
This sentence ends with ***bold italic***.
```

This input, from `tests/corpus/core/inline/strong-emphasis.etch`, produces `Strong` nodes that each contain one nested `Emphasis` node.

## 5.4 Strikethrough

### Structure

1. **Opening delimiter**: Two consecutive tildes: `~~`.
2. **Closing delimiter**: Two consecutive tildes: `~~`.
3. **Flanking rules**: The opening `~~` is valid only when followed by a non-whitespace character. The closing `~~` is valid only when preceded by a non-whitespace character. `~~`, `~~ ~~`, `~~ text~~`, and `~~text ~~` MUST remain literal text.
4. **Content model**: Strikethrough content is parsed recursively as inline content.
5. **Nesting**: Strikethrough MAY contain other inline forms.
6. **Precedence**: At a run of tildes, the parser MUST try `~~` strikethrough before single-tilde subscript. A run of exactly two tildes is therefore never parsed as two subscript delimiters.

### Normative Examples

`✓ Basic`

```etch
~~Struck~~ opens this sentence.
This sentence has ~~struck~~ in the middle.
This sentence ends with ~~struck~~.
```

This input, from `tests/corpus/core/inline/strikethrough.etch`, produces `Strikethrough` nodes in each position.

`✓ Edge case`

```etch
~~Strike starts here
and ends here~~ at the end of the paragraph.
```

This input, from `tests/corpus/core/inline/across-lines.etch`, produces one `Strikethrough` node containing a `SoftBreak`.

## 5.5 Inline Code

### Structure

1. **Opening delimiter**: Either one backtick (`` ` ``) or two backticks (`` `` ``).
2. **Closing delimiter**: For a one-backtick opener, the closing delimiter is the next backtick run of length at least one. For a two-backtick opener, the closing delimiter is the next backtick run of length at least two. Runs of three or more backticks are not valid opening delimiters for inline code.
3. **Flanking rules**: Inline code spans have no whitespace-based flanking rule. The opening run is valid whenever it is a run of length one or two and a later closing run of sufficient length exists.
4. **Content model**: Inline code content is raw literal text. No inline formatting, autolink recognition, link parsing, image parsing, footnote parsing, hard-break recognition, soft-break conversion, or backslash escaping is processed inside the code span. Newlines inside the span are preserved as literal newline characters in the code value.
5. **Nesting**: No inline syntax nests inside inline code. Backticks inside the content are allowed only when the opener length is sufficient to avoid premature closure.
6. **Precedence**: At a backtick run, the parser chooses the opener length by the exact run length at the current position. Because inline code is recognized before delimiter-based spans, characters such as `*` or `~~` inside a code span never begin formatting.

### Normative Examples

`✓ Basic`

```etch
`printf()` opens this sentence.
This sentence uses `npm test` in the middle.
```

This input, from `tests/corpus/core/inline/inline-code.etch`, produces `InlineCode` nodes with the literal values `printf()` and `npm test`.

`✓ Edge case`

```etch
This sentence ends with ``code containing `backticks` inside``.
```

This input, from `tests/corpus/core/inline/inline-code.etch`, produces one `InlineCode` node whose literal value is `code containing `backticks` inside`.

## 5.6 Superscript

### Structure

1. **Opening delimiter**: A single caret: `^`.
2. **Closing delimiter**: A single caret: `^`.
3. **Flanking rules**: The opening `^` is valid only when followed by a non-whitespace character. The closing `^` is valid only when preceded by a non-whitespace character. `^ ^`, `^ text^`, and `^text ^` MUST remain literal text.
4. **Content model**: Superscript content is parsed recursively as inline content.
5. **Nesting**: Superscript MAY contain the other inline forms defined by this chapter.
6. **Precedence**: Only a run of exactly one caret is recognized. `^^` and longer caret runs MUST remain literal text.

### Normative Examples

`✓ Basic`

```etch
^Start^ opens this sentence.
This sentence includes E = mc^2^ in the middle.
This line ends with 1^st^.
```

This input, from `tests/corpus/core/inline/superscript.etch`, produces `Superscript` nodes around `Start`, `2`, and `st`.

`✓ Edge case`

```etch
These tokens stay literal too: ~~ and ^^ in plain text.
```

This line, from `tests/corpus/core/inline/empty-markers.etch`, demonstrates that `^^` is not a valid superscript delimiter run and remains literal text.

## 5.7 Subscript

### Structure

1. **Opening delimiter**: A single tilde: `~`.
2. **Closing delimiter**: A single tilde: `~`.
3. **Flanking rules**: The opening `~` is valid only when followed by a non-whitespace character. The closing `~` is valid only when preceded by a non-whitespace character. `~ ~`, `~ text~`, and `~text ~` MUST remain literal text.
4. **Content model**: Subscript content is parsed recursively as inline content.
5. **Nesting**: Subscript MAY contain the other inline forms defined by this chapter.
6. **Precedence**: At a tilde run, `~~` strikethrough is recognized before single-tilde subscript. Single-tilde subscript is considered only when the run length is exactly one.

### Normative Examples

`✓ Basic`

```etch
~Start~ opens this sentence.
This sentence includes H~2~O in the middle.
This line ends with CO~2~.
```

This input, from `tests/corpus/core/inline/subscript.etch`, produces `Subscript` nodes around `Start` and both occurrences of `2`.

## 5.8 Highlight

### Structure

1. **Opening delimiter**: Two consecutive equals signs: `==`.
2. **Closing delimiter**: Two consecutive equals signs: `==`.
3. **Flanking rules**: The opening `==` is valid only when followed by a non-whitespace character. The closing `==` is valid only when preceded by a non-whitespace character. `==`, `== ==`, `== text==`, and `==text ==` MUST remain literal text.
4. **Content model**: Highlight content is parsed recursively as inline content.
5. **Nesting**: Highlight MAY contain the other inline forms defined by this chapter.
6. **Precedence**: Only a run of exactly two equals signs is recognized. Any other equals-sign run length remains literal text.

### Normative Examples

`✓ Basic`

```etch
==Highlighted== opens this sentence.
This sentence has ==highlighted== in the middle.
This sentence ends with ==highlighted==.
```

This input, from `tests/corpus/core/inline/highlight.etch`, produces `Highlight` nodes in each position.

`✓ Edge case`

```etch
This line has ==mark==++insert++~~gone~~ in the middle.
```

This line, from `tests/corpus/core/inline/adjacent.etch`, demonstrates that a highlight span may end immediately before an insert span and a strikethrough span without intervening whitespace.

## 5.9 Insert

### Structure

1. **Opening delimiter**: Two consecutive plus signs: `++`.
2. **Closing delimiter**: Two consecutive plus signs: `++`.
3. **Flanking rules**: The opening `++` is valid only when followed by a non-whitespace character. The closing `++` is valid only when preceded by a non-whitespace character. `++`, `++ ++`, `++ text++`, and `++text ++` MUST remain literal text.
4. **Content model**: Insert content is parsed recursively as inline content.
5. **Nesting**: Insert MAY contain the other inline forms defined by this chapter.
6. **Precedence**: Only a run of exactly two plus signs is recognized. `++++` is not an insert opener and remains literal text.

### Normative Examples

`✓ Basic`

```etch
++Inserted++ opens this sentence.
This sentence has ++inserted++ in the middle.
This sentence ends with ++inserted++.
```

This input, from `tests/corpus/core/inline/insert.etch`, produces `Insert` nodes in each position.

`✓ Edge case`

```etch
These tokens also stay literal: == and ++++ in plain text.
```

This line, from `tests/corpus/core/inline/empty-markers.etch`, demonstrates that `++++` is not parsed as an insert span.

## 5.10 Links

### Structure

1. **Opening delimiter**: A left bracket: `[`, provided it is not immediately preceded by `!`.
2. **Closing delimiter**: Link text closes at the matching balanced `]`. A link is recognized only when that `]` is followed immediately by `(` and the destination then closes at the matching balanced `)`.
3. **Flanking rules**: Links use no whitespace-based flanking rule. The bracketed text MAY contain whitespace. The parenthesized destination, after trimming surrounding whitespace, MUST contain a non-empty URL. If a title is present, it MUST be separated from the URL by whitespace and MUST occupy the entire remaining destination as one double-quoted string.
4. **Content model**: The link text between `[` and `]` is parsed recursively as inline content. The destination URL and optional title are raw strings and are not inline-parsed. Balanced bracket counting applies within the link text, and balanced parenthesis counting applies within the destination.
5. **Nesting**: Because link text is parsed with the inline parser, other inline forms MAY appear inside link text. The approved corpus exercises `Strong` and `Emphasis` inside links.
6. **Precedence**: At `[`, the parser MUST try footnote-reference parsing first when the next character is `^`; otherwise it tries link parsing. `![` begins image parsing and is recognized before bare `[` links.

### Normative Examples

`✓ Basic`

```etch
Read the [Etch getting started guide](https://docs.etch-lang.dev/guide/getting-started) before writing your first document.
```

This input, from `tests/corpus/core/links-images/basic-link.etch`, produces one `Link` node whose text content is `Etch getting started guide`.

`✓ Edge case`

```etch
Browse the [Etch syntax reference](https://docs.etch-lang.dev/reference/core-syntax "Core syntax reference") for the full block and inline rules.
```

This input, from `tests/corpus/core/links-images/link-with-title.etch`, produces one `Link` node with URL `https://docs.etch-lang.dev/reference/core-syntax` and title `Core syntax reference`.

`✓ Edge case`

```etch
Open the [**Etch** *quickstart* guide](https://docs.etch-lang.dev/guide/quickstart?view=full#first-document) to see formatted link text in one link.
```

This input, from `tests/corpus/core/links-images/link-with-formatting.etch`, demonstrates that link text is itself parsed as inline content.

## 5.11 Images

### Structure

1. **Opening delimiter**: An exclamation mark followed immediately by a left bracket: `![`.
2. **Closing delimiter**: The alt text closes at the matching balanced `]`. An image is recognized only when that `]` is followed immediately by `(` and the destination then closes at the matching balanced `)`. An optional attribute set MAY follow immediately after `)` with no intervening whitespace.
3. **Flanking rules**: Images use no whitespace-based flanking rule. The destination, after trimming surrounding whitespace, MUST contain a non-empty URL. If a title is present, it MUST be separated from the URL by whitespace and MUST occupy the entire remaining destination as one double-quoted string. If attributes are present, the first character after `)` MUST be `{`.
4. **Content model**: The alt text between `[` and `]` is a literal string, not recursively parsed inline content. The URL, optional title, and optional attribute set are also not inline-parsed.
5. **Nesting**: Inline syntax does not nest inside image alt text. An image node itself MAY appear anywhere inline content may appear.
6. **Precedence**: `![` MUST be parsed as an image opener before the parser considers the following `[` as a link opener.

### Normative Examples

`✓ Basic`

```etch
![Snowy trail at dawn](images/snowy-trail.jpg)

The travel note includes ![campfire icon](icons/campfire.png) beside the route summary.
```

This input, from `tests/corpus/core/links-images/basic-image.etch`, demonstrates that images may appear as the sole inline content of a paragraph or inside a larger text run.

`✓ Edge case`

```etch
![River overlook](photos/river-overlook.jpg "Morning light over the valley")
```

This input, from `tests/corpus/core/links-images/image-with-title.etch`, produces one `Image` node with title `Morning light over the valley`.

`✓ Edge case`

```etch
![Author portrait](photos/author-portrait.jpg){width=80% .rounded}
```

This input, from `tests/corpus/core/links-images/image-with-attrs.etch`, demonstrates the immediate trailing attribute form on images.

## 5.12 Autolinks

### Structure

1. **Opening delimiter**: The literal prefix `http://` or `https://`.
2. **Closing delimiter**: None. The autolink ends immediately before the first whitespace character after the scheme or at end of input.
3. **Flanking rules**: Autolinks use no whitespace-based flanking rule. A recognized scheme MUST appear exactly at the current parse position, and at least one non-whitespace character MUST follow the scheme.
4. **Content model**: An autolink produces one `AutoLink` node containing the literal URL string. It has no child inline content.
5. **Nesting**: Autolinks do not contain nested inline syntax, but they MAY appear alongside other inline forms in the surrounding content.
6. **Precedence**: Autolinks are recognized before images, links, inline code, and delimiter-based spans whenever the current character is `h`.

### Normative Examples

`✓ Basic`

```etch
Visit https://example.com/help/getting-started?lang=en#overview for details.
Check http://docs.etch-lang.dev/guide for the older guide.
```

This input, from `tests/corpus/core/links-images/autolink.etch`, produces two `AutoLink` nodes, one for the `https://` URL and one for the `http://` URL.

`✓ Edge case`

```etch
The bare domain example.com should stay plain text.
The file is at ftp://files.etch-lang.dev/releases/latest.zip and should stay plain text.
```

These lines, from `tests/corpus/core/links-images/autolink.etch`, demonstrate that bare domains and non-HTTP schemes are not autolinks.

## 5.13 Footnote References

### Structure

1. **Opening delimiter**: A left bracket followed immediately by a caret: `[^`.
2. **Closing delimiter**: The first subsequent right bracket: `]`.
3. **Flanking rules**: Footnote references use no whitespace-based flanking rule. The label between `[^` and `]` MUST be non-empty.
4. **Content model**: The label is literal text. It is not parsed as inline content and does not use balanced-bracket matching.
5. **Nesting**: Footnote-reference labels do not contain nested inline syntax. A `FootnoteReference` node MAY appear anywhere inline content may appear.
6. **Precedence**: At `[`, footnote-reference parsing is attempted before ordinary link parsing. Therefore `[^label]` is always a footnote reference, never the text portion of a link.

### Normative Examples

`✓ Basic`

```etch
The old station still keeps a brass clock above the ticket window.[^1]
```

This input, from `tests/corpus/core/footnotes/basic.etch`, produces one `FootnoteReference` node with label `1`.

`✓ Edge case`

```etch
The final page includes a note about the missing lantern.[^my-note]
```

This input, from `tests/corpus/core/footnotes/named.etch`, demonstrates that labels are not limited to digits.

## 5.14 Soft Breaks

### Structure

1. **Opening delimiter**: None. A soft break is produced by a physical line ending inside an inline-bearing block when hard-break recognition has not already consumed that boundary.
2. **Closing delimiter**: None.
3. **Flanking rules**: Soft breaks use no whitespace-based flanking rule. They are produced from `LF`, `CRLF`, or `CR` line endings.
4. **Content model**: A soft break produces one `SoftBreak` inline node.
5. **Nesting**: Soft breaks MAY occur inside recursively parsed inline content. The approved corpus exercises soft breaks both in ordinary paragraphs and inside delimiter-based spans that continue across lines.
6. **Precedence**: Hard-break recognition from a trailing backslash is attempted first. If no hard break is recognized, the line ending becomes a soft break.

### Normative Examples

`✓ Basic`

```etch
This paragraph starts on one line
continues on the next line
and ends on a third line.
```

This input, from `tests/corpus/core/paragraphs/multi-line.etch`, produces one paragraph containing two `SoftBreak` nodes.

`✓ Edge case`

```etch
*Starts on this line
and ends on the next* in the same paragraph.
```

This input, from `tests/corpus/core/inline/across-lines.etch`, demonstrates that a soft break may occur inside an inline span rather than ending the span.

## 5.15 Hard Breaks

### Structure

1. **Opening delimiter**: A backslash immediately followed by a line ending: `\\\n` or `\\\r\n`.
2. **Closing delimiter**: None. The backslash and the following line ending are consumed together.
3. **Flanking rules**: The backslash MUST be the final character on the source line. A non-trailing backslash is not a hard break. Two trailing spaces MUST NOT create a hard break.
4. **Content model**: A hard break produces one `HardBreak` inline node.
5. **Nesting**: Hard breaks are recognized by the recursive inline parser and MAY therefore appear inside any inline-bearing context, including formatted spans.
6. **Precedence**: At `\`, hard-break recognition is attempted before ordinary backslash escaping and before the following newline can be interpreted as a soft break.

### Normative Examples

`✓ Basic`

```etch
123 Main Street\
Apartment 4B\
New York, NY 10001
```

This input, from `tests/corpus/core/hard-line-breaks/backslash.etch`, produces one paragraph containing two `HardBreak` nodes.

`✓ Edge case`

```etch
These lines end with two trailing spaces.  
They should still stay in one paragraph.  
No hard break is created unless a backslash ends the line.
```

This input, from `tests/corpus/core/hard-line-breaks/two-spaces-no-break.etch`, demonstrates that trailing spaces produce `SoftBreak` nodes rather than `HardBreak` nodes.

## 5.16 Escaping

### Structure

1. **Opening delimiter**: A backslash: `\`.
2. **Closing delimiter**: None. The backslash escapes exactly one following character when that character is escapable.
3. **Flanking rules**: Escapes use no whitespace-based flanking rule. A backslash escape is recognized only when the next character is one of `*`, `~`, `^`, `=`, `+`, `[`, `]`, or `\`. If the next character is not in that set, the backslash remains literal unless it forms a hard break.
4. **Content model**: An escape contributes the escaped character as literal text and prevents it from starting or closing inline syntax.
5. **Nesting**: Escapes are recognized during ordinary inline parsing, including inside delimiter-based spans and link text. Escapes are not processed inside inline code spans.
6. **Precedence**: At `\`, hard-break recognition is attempted first. If no hard break is recognized, escape recognition is attempted next.

### Normative Examples

`✓ Basic`

```etch
\*not italic\* stays literal at the start.
This sentence keeps \~not sub\~, \^not super\^, and \=\=not highlight\=\= literal.
This sentence ends with \+\+not insert\+\+.
```

This input, from `tests/corpus/core/inline/escaped.etch`, demonstrates that escaped formatting markers remain literal text rather than opening inline spans.

## 5.17 Delimiter Disambiguation Summary

The following rules are normative and summarize the most important delimiter conflicts defined above:

1. `~~` is parsed as strikethrough before single-tilde subscript is considered.
2. `***` is parsed as strong+emphasis before `**` or `*` parsing is considered.
3. `[^` begins a footnote reference before ordinary link parsing is considered.
4. `![` begins an image before ordinary link parsing is considered.
5. Empty delimiter pairs such as `**`, `~~`, `^^`, `==`, and `++++` remain literal text.
6. Inside inline code, no formatting or escaping is processed.

`✓ Example`

```etch
These tokens stay literal: ** and **** in plain text.
These tokens stay literal too: ~~ and ^^ in plain text.
These tokens also stay literal: == and ++++ in plain text.
```

This input, from `tests/corpus/core/inline/empty-markers.etch`, exercises the literal fallback for empty and unsupported delimiter runs.
