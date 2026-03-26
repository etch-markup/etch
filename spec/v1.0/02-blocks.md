# 4. Block Syntax

This chapter defines the core block-level constructs of Etch. After frontmatter and comments are handled as defined elsewhere, the parser MUST read the remaining document as an ordered sequence of block elements.

Block recognition is ordered. A conforming parser MUST recognize a more specific block construct before falling back to a paragraph. Within the core constructs covered here, paragraphs are therefore the final fallback.

## 4.1 Paragraphs

### Structure

1. **Recognition rule**: A paragraph begins at the first non-blank line that is not recognized as another block construct. A paragraph therefore consists of one or more consecutive non-blank lines collected by fallback parsing.
2. **Content model**: A paragraph contains inline content only. A physical line break inside a paragraph produces a `SoftBreak` inline node unless the line ends with a trailing backslash, in which case it produces a `HardBreak`.
3. **Termination rule**: A paragraph ends immediately before a blank line, immediately before a following line that begins another block construct, or at end of input.
4. **Attributes**: A paragraph MAY carry a trailing attribute set. The attribute set MUST appear on the paragraph's final source line, MUST be separated from the preceding inline content by whitespace, and MUST be removed from the paragraph text before inline parsing.

### Normative Examples

`✓ Basic`

```etch
This is one paragraph on a single line.
```

This input produces one `Paragraph` block containing one `Text` node.

`✓ Edge case`

```etch
This paragraph starts on one line
continues on the next line
and ends on a third line.
```

This input produces one `Paragraph`. The line boundaries remain inside the paragraph as `SoftBreak` inline nodes.

`✗ Negative`

```etch
# Heading
```

This line does not produce a paragraph. It produces a `Heading`, because headings are recognized before paragraph fallback.

## 4.2 Headings

### Structure

1. **Recognition rule**: A heading is recognized only when a line begins with between one and six `#` characters followed immediately by a space (`U+0020`). One `#` produces level 1; six `#` characters produce level 6.
2. **Content model**: A heading contains inline content only.
3. **Termination rule**: A heading consumes exactly one source line.
4. **Attributes**: A heading MAY carry a trailing attribute set on the same line. The attribute set MUST follow the heading content, MUST be separated from that content by whitespace, and MUST be removed before inline parsing.

### Normative Examples

`✓ Basic`

```etch
## Immediate paragraph heading
This paragraph starts right after the heading with no blank line.
```

The first line produces a level-2 `Heading`. The following line produces a `Paragraph`; a blank line is not required after a heading.

`✓ Edge case`

```etch
# A **bold** heading
```

This input produces a level-1 `Heading` whose inline content contains `Text`, `Strong`, and `Text` nodes.

`✗ Negative`

```etch
#no-space
####### Too many hashes for a valid heading
```

Neither line produces a heading. The first line lacks the required space after `#`, and the second line begins with seven `#` characters. Both lines therefore fall back to paragraph parsing.

## 4.3 Thematic Breaks

### Structure

1. **Recognition rule**: A thematic break is recognized when, after trimming leading and trailing spaces, a line consists only of one marker character repeated at least three times, optionally separated by spaces. The marker character MUST be all `-`, all `*`, or all `_`; mixed marker characters MUST NOT match.
2. **Content model**: A thematic break contains no child content.
3. **Termination rule**: A thematic break consumes exactly one source line.
4. **Attributes**: No thematic-break attribute placement is exercised by the approved corpus. This chapter therefore defines no dedicated source form for attaching attributes to a thematic break.

### Normative Examples

`✓ Basic`

```etch
The campfire burned low as the guide folded the map.

---

By morning the ashes were hidden under fresh snow.
```

The middle line produces one `ThematicBreak` block between two paragraphs.

`✓ Edge case`

```etch
# Winter Notes

This document starts with a heading, so frontmatter is no longer possible.

---

The line above must parse as a thematic break in the middle of the document.
```

The `---` line produces a `ThematicBreak`, because content has already begun and frontmatter is no longer possible.

`✗ Negative`

```etch
---
```

When this exact line occurs as the first line of the document body, it MUST NOT be recognized as a thematic break. It is reserved for frontmatter recognition; if no frontmatter block is completed, the line falls back to paragraph text.

## 4.4 Fenced Code Blocks

### Structure

1. **Recognition rule**: A fenced code block opens on a line whose first three characters are `` ``` ``. Any remaining characters on that opening line, after trimming surrounding whitespace, form the optional language identifier. If the trimmed remainder is empty, the block has no language identifier.
2. **Content model**: A fenced code block contains literal raw text. Inline parsing, block parsing, comment parsing, directive parsing, and attribute parsing MUST NOT occur inside the code block body.
3. **Termination rule**: A fenced code block ends at the next later line that is exactly `` ``` ``. If no such line appears, the block consumes the remainder of the input.
4. **Attributes**: A fenced code block MAY carry an attribute set on the immediately following attribute-only line after its closing fence.

### Normative Examples

`✓ Basic`

````etch
```rust
fn main() {
    let title = "Etch";
    println!("Hello from {title}");
}
```
````

This input produces one `CodeBlock` with language `rust`. The body content is the literal text between the fences.

`✓ Edge case`

````etch
```
status = "**draft**"
:note[remember this]
{~ hidden in code ~}
```
````

This input produces one `CodeBlock`. The strings that look like emphasis, directives, and comments remain literal code and are not parsed as Etch markup.

`✗ Negative`

```etch
``
not a fenced code block
``
```

This input does not produce a fenced code block, because the opening and closing lines do not begin with the required three-character fence `` ``` ``.

## 4.5 Blockquotes

### Structure

1. **Recognition rule**: A blockquote line is recognized only when its first character is `>`. The parser MUST remove the first `>` and MAY remove one immediately following space. The resulting text is the quoted source for that line.
2. **Content model**: A blockquote contains a recursively parsed sequence of block elements. Nested blockquotes arise when the stripped quoted content itself begins with `>`.
3. **Termination rule**: A blockquote consists of one or more consecutive lines beginning with `>`. A blank line belongs to the blockquote only if that blank line is itself quoted as `>` or `> `.
4. **Attributes**: No dedicated quote-level attribute placement is exercised by the approved corpus. In the approved corpus, a trailing attribute set written inside quoted content attaches to the contained block produced from that content, not to the outer `BlockQuote` node itself.

### Normative Examples

`✓ Basic`

```etch
> The road vanished under the first snow.
> Lantern light flickered across the fence posts.
> Nobody spoke until the wind settled.
```

This input produces one `BlockQuote` containing one paragraph. The quoted source lines remain within that paragraph as `SoftBreak` nodes.

`✓ Edge case`

```etch
> Outer voice speaking to the room.
>> Inner reply from the back row.
>>> A quiet aside beneath both speakers.
>> The reply continues after the aside.
```

This input produces a `BlockQuote` containing a nested `BlockQuote`, which itself contains a further nested `BlockQuote`.

`✗ Negative`

```etch
 > The marker is not in column 1.
```

This line does not produce a blockquote, because the first character is a space rather than `>`. It therefore falls back to paragraph parsing.

## 4.6 Lists

### Structure

1. **Recognition rule**: A list item begins with one of the following markers at the current list position:
   1. `- ` for an unordered item;
   2. `- [x] ` for a checked task item;
   3. `- [ ] ` for an unchecked task item; or
   4. one or more ASCII digits followed by `. ` for an ordered item.
   Task recognition is case-sensitive: only lowercase `x` and a literal space are valid task markers.
2. **Content model**: A list contains one or more list items. Each list item contains a sequence of block elements. The text after the item marker begins the first block of the item. Additional item content comes from continuation lines indented by at least two spaces more than the item's own indentation. Continuation content is recursively parsed, so multi-paragraph items, nested lists, and nested blockquotes are permitted.
3. **Termination rule**: A list ends before the first line that is neither:
   1. a sibling list item of the same marker family at the current list level; nor
   2. an indented continuation line belonging to the current item.
   A line indented by two or more spaces beyond its parent list level begins one nesting level deeper. The parser is lenient: any indentation of at least two spaces counts as one level deeper.
4. **Attributes**: No list-level attribute placement is exercised by the approved corpus. Attributes written inside list items attach to the blocks produced within those items.

### Normative Examples

`✓ Basic`

```etch
1. Preheat the oven to 400F
2. Chop the carrots and onions
3. Season the stock with thyme
4. Simmer the soup for thirty minutes
```

This input produces one ordered `List` containing four items.

`✓ Edge case`

```etch
- Camp briefing for the new arrivals.
  Bring dry socks, a flashlight, and a printed map.

  Check in at the ranger station before sunset.
```

This input produces one unordered list item containing two paragraphs. The second and third physical lines belong to the same list item because they are indented by at least two spaces beyond the list marker.

`✓ Edge case`

```etch
- [x] Reserve the campsite
- [ ] Print the route map
```

This input produces one unordered `List`. The first item has task state `checked`; the second item has task state `unchecked`.

`✗ Negative`

```etch
-item without required space
1.step without required space
```

Neither line begins a list item. Both lines lack the required space after the marker and therefore fall back to paragraph parsing.

## 4.7 Tables

### Structure

1. **Recognition rule**: A table begins only when all of the following are true:
   1. the current line, after trimming surrounding spaces, begins with `|` and ends with `|`;
   2. splitting that trimmed line on `|` yields one or more header cells; and
   3. the immediately following line is a separator row with the same number of cells, where each cell is one of:
      1. `---...` for no alignment;
      2. `:---...` for left alignment;
      3. `:---...:` for center alignment; or
      4. `---...:` for right alignment.
   In every separator cell, the sequence between optional colons MUST contain at least three `-` characters and MUST contain no other characters.
2. **Content model**: A table contains header cells, zero or more data rows, and one alignment value per column. Table cells contain inline content only.
3. **Termination rule**: After the separator row, subsequent lines belong to the table only while they remain pipe-delimited rows with exactly the same number of cells as the header row. The first non-matching line ends the table and is parsed normally as the next block.
4. **Attributes**: A table MAY carry an attribute set on the immediately following attribute-only line after its last row.

### Normative Examples

`✓ Basic`

```etch
| Trail | Distance |
| --- | --- |
| Pine Loop | 4 km |
| Ridge Walk | 7 km |
| River Path | 3 km |
```

This input produces one `Table` with two header cells, three data rows, and alignment `None` for both columns.

`✓ Edge case`

```etch
| Item | Status | Count |
| :--- | :---: | ---: |
| Lanterns | Ready | 12 |
| Blankets | Packed | 8 |
| Maps | Missing | 1 |
```

This input produces one `Table` whose columns are aligned left, center, and right respectively.

`✗ Negative`

```etch
| Header |
| not a separator |
```

This input does not produce a table, because the second line is not a valid separator row.

## 4.8 Footnote Definitions

### Structure

1. **Recognition rule**: A footnote definition begins on a line that starts with `[^`, contains a non-empty label, and then contains the literal delimiter `]:`. The substring between `[^` and `]:` is the footnote label. The content after `]:` begins the footnote body.
2. **Content model**: A footnote definition contains a sequence of block elements. The opening line contributes the first content of the first block. Additional content MAY come from:
   1. continuation lines indented by at least two spaces; and
   2. an immediate unindented lazy continuation line, provided no blank line intervenes and the line cannot begin another block construct.
3. **Termination rule**: A footnote definition ends before the first subsequent line that is not part of the footnote body. After a blank line, continuation content MUST be indented by at least two spaces to remain inside the same footnote definition.
4. **Attributes**: No dedicated footnote-definition attribute placement is exercised by the approved corpus.

### Normative Examples

`✓ Basic`

```etch
The old station still keeps a brass clock above the ticket window.[^1]

[^1]: The clock was installed in 1912 and still runs by hand.
```

This input produces one paragraph containing a `FootnoteReference` and one `FootnoteDefinition` with label `1`.

`✓ Edge case`

```etch
The village record mentions the flood only once.[^long-note]

[^long-note]: The clerk added the note after the council meeting
  and copied it into the winter ledger the same night.

  A second paragraph explains that the bridge repairs lasted
  until the first week of spring.
```

This input produces one `FootnoteDefinition` with two paragraphs. The indented continuation lines remain inside the footnote body.

`✗ Negative`

```etch
[note]: Not a footnote definition.
[^]: Empty labels are invalid.
```

Neither line produces a footnote definition. The first line lacks the required `^`, and the second line uses an empty label.

## 4.9 Definition Lists

### Structure

1. **Recognition rule**: A definition list begins only when a non-blank term line is immediately followed by a definition opening line that begins with `: `.
2. **Content model**: A definition list contains one or more items. Each item contains:
   1. one term, parsed as inline content from the term line; and
   2. one or more definitions.
   Each definition begins with the content following `: ` on its opening line and MAY continue through indented continuation lines, which are recursively parsed as block content.
3. **Termination rule**: A definition continues while subsequent non-blank lines are indented by at least two spaces. A definition list continues with a new item only when, after any intervening blank lines, the next non-blank line is a term line that is itself immediately followed by another `: ` definition opening line.
4. **Attributes**: No definition-list attribute placement is exercised by the approved corpus.

### Normative Examples

`✓ Basic`

```etch
Lantern
: A portable light used on the trail after sunset.
```

This input produces one `DefinitionList` containing one term and one definition.

`✓ Edge case`

```etch
Draft
: A current of cold air near a door or window.
: An unfinished version of a document.
```

This input produces one `DefinitionList` item with one term and two separate definitions.

`✗ Negative`

```etch
:not a definition
```

This line does not open a definition, because a definition opening requires `: ` with a space after the colon. The line therefore falls back to paragraph parsing.

## 4.10 Hard Line Breaks in Block Content

Hard line breaks are inline nodes, but their recognition affects how multi-line block content is interpreted.

### Structure

1. **Recognition rule**: A hard line break is recognized only when a source line inside paragraph-like inline content ends with a trailing backslash (`\`) immediately before the line ending.
2. **Content model**: A hard line break produces a `HardBreak` inline node between the text on the current line and the text on the next line.
3. **Termination rule**: A hard line break does not end the surrounding block. It keeps the surrounding paragraph-like block content contiguous across the line boundary.
4. **Attributes**: Hard line breaks do not carry attributes.

### Normative Examples

`✓ Basic`

```etch
123 Main Street\
Apartment 4B\
New York, NY 10001
```

This input produces one paragraph containing two `HardBreak` inline nodes.

`✓ Edge case`

```etch
> The snow started before dawn\
> and kept falling through breakfast\
> until the road signs disappeared.
```

This input produces one blockquote containing a paragraph with two `HardBreak` inline nodes.

`✗ Negative`

```etch
These lines end with two trailing spaces.  
They should still stay in one paragraph.  
No hard break is created unless a backslash ends the line.
```

This input does not produce any `HardBreak` nodes. The physical line boundaries remain `SoftBreak` nodes because two trailing spaces MUST NOT create a hard line break in Etch.
