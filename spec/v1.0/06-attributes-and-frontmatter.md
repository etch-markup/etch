# 8. Attributes and Frontmatter

This chapter defines Etch attribute sets and YAML frontmatter. Attribute parsing and frontmatter recognition are normative. A conforming parser MUST apply the rules in this chapter exactly as written.

## 8.1 Attributes

### Structure

1. **Recognition rule**: An attribute set begins with `{` immediately followed by `#`, `.`, or a letter. `{~` does not begin an attribute set; it begins a comment as defined in Chapter 6.
2. **Delimiters**: An attribute set is enclosed in `{` and `}`.
3. **Attribute entries**: Inside one attribute set, entries are separated by spaces. The supported entry forms are `#id`, `.class`, `key=value`, and `key="quoted value"`.
4. **Identifier form**: `#id` assigns an identifier to the modified element.
5. **Class form**: `.class` adds one class to the modified element. Multiple class entries are allowed in the same attribute set.
6. **Unquoted key-value form**: `key=value` assigns an unquoted attribute value.
7. **Quoted key-value form**: `key="quoted value"` assigns a quoted attribute value. Quoted values MAY contain spaces.
8. **Escaped quote**: Inside a quoted attribute value, a literal double quote MUST be written as `\"`.
9. **Attachment rule**: An attribute set modifies the immediately preceding element.
10. **Single-line attachment**: For single-line elements such as headings, paragraphs, and images, the attribute set appears on the same source line as the element it modifies.
11. **Block attachment**: For multi-line block elements such as fenced code blocks and tables, the attribute set appears after the block it modifies.
12. **Comment disambiguation**: Because `{~` begins a comment, a parser MUST distinguish comment syntax from attribute syntax by the first character after `{`.
13. **Repeated classes**: When more than one `.class` entry appears in the same attribute set, all such classes apply.
14. **Repeated identifiers**: When more than one `#id` entry appears in the same attribute set, the last `#id` entry wins.

### Normative Examples

`✓ Individual forms`

```etch
# Title {#my-id}

A centered announcement for the station platform. {.centered}

Reference card {key=value}

Reference card {key2="quoted value with spaces"}
```

These examples demonstrate the four supported attribute entry forms: `#id`, `.class`, `key=value`, and `key="quoted value with spaces"`.

`✓ Combined attribute set`

```etch
Reference card {#my-id .class1 .class2 key=value key2="quoted value with spaces"}
```

This input, from `tests/corpus/extensions/attributes/combined.etch`, demonstrates that one attribute set may combine identifier, multiple class, unquoted key-value, and quoted key-value entries.

`✓ Escaped quote inside a quoted value`

```etch
Quoted label {key="value with \"escaped quotes\" inside"}
```

This input, from `tests/corpus/extensions/attributes/escaped-quote.etch`, demonstrates the required escape form for a literal `"` inside a quoted attribute value.

`✓ Same-line attachment on a heading`

```etch
# Title {#my-id}
## Subtitle {.special-class lang=en}
```

This input, from `tests/corpus/extensions/attributes/on-heading.etch`, demonstrates same-line attachment on headings.

`✓ Same-line attachment on an image`

```etch
![Trail marker](photo.jpg){width=80% .rounded}
```

This input, from `tests/corpus/extensions/attributes/on-image.etch`, demonstrates same-line attachment on an image element.

`✓ Attachment to the immediately preceding line inside a blockquote`

```etch
> The harbor remembers every winter storm.
> — Elias North {.attribution}
```

This input, from `tests/corpus/extensions/attributes/on-blockquote.etch`, demonstrates that an attribute set attaches to the immediately preceding element, here the attribution line inside a blockquote.

`✓ Block attachment after a fenced code block`

````etch
```rust
fn main() {
    println!("Etch");
}
```
{.line-numbers highlight=3}
````

This input, from `tests/corpus/extensions/attributes/on-code-block.etch`, demonstrates that a fenced code block's attributes appear after the block.

`✓ Block attachment after a table`

```etch
| Stop | Time |
| --- | --- |
| North Gate | 08:15 |
| River Dock | 09:40 |
{.striped .compact}
```

This input, from `tests/corpus/extensions/attributes/on-table.etch`, demonstrates that a table's attributes appear after the table block.

`✓ Comment and attributes are distinct`

```etch
# Title {~ note to self ~} {#my-id}
```

This example, also covered by Chapter 6, demonstrates that `{~ ... ~}` is parsed as a comment, while the following `{#my-id}` remains an attribute set.

`✓ Multiple classes, last id wins`

```etch
# Title {#draft #final .featured .compact}
```

This heading has identifier `final` and classes `featured` and `compact`. The final `#id` entry overrides the earlier one, while both class entries remain in effect.

## 8.2 Frontmatter

### Structure

1. **Opening position**: Frontmatter begins only when a line containing exactly `---` appears on the very first line of the file.
2. **Strict positioning**: If any blank line or any other content appears before `---`, then that `---` is not frontmatter.
3. **Closing fence**: A frontmatter block closes at the next later line containing exactly `---`.
4. **Content type**: The content between the opening and closing fences is YAML and MUST be parsed by a YAML parser.
5. **Field shapes**: Frontmatter MAY contain arbitrary YAML values, including strings, numbers, booleans, arrays, and nested objects.
6. **Empty frontmatter**: An empty frontmatter block is valid. `---` immediately followed by `---` therefore represents a frontmatter block with empty YAML content.
7. **Not a late construct**: A `---` line that is not on the first line of the file is never frontmatter. In that position it is a thematic break, as defined in Chapter 4.
8. **Resume document parsing**: After the closing `---`, normal Etch block parsing resumes with the following line.

### Normative Examples

`✓ Basic frontmatter`

```etch
---
title: "Winter Notes"
author: trailwriter
---

The lantern log begins after a simple frontmatter block.
```

This input, from `tests/corpus/extensions/frontmatter/basic.etch`, demonstrates a minimal frontmatter block at the top of the file.

`✓ Frontmatter with various YAML types`

```etch
---
title: "Chronicle of the Ridge"
author: "foxwriter42"
chapter: 7
tags: [travel, winter, field-notes]
series:
  name: "Northern Passages"
  part: 3
draft: true
word_count: auto
---

This document starts after a frontmatter block using several YAML field shapes.
```

This input, from `tests/corpus/extensions/frontmatter/full.etch`, demonstrates strings, numbers, arrays, booleans, and nested objects in frontmatter content.

`✓ Empty frontmatter`

```etch
---
---

This document has empty frontmatter and then normal content.
```

This input, from `tests/corpus/extensions/frontmatter/empty.etch`, demonstrates that empty frontmatter is valid.

`✗ Not frontmatter when not on the first line`

```etch

---

{~ note: --- is only frontmatter on the very first line ~}

This file starts with a blank line, so the dashes above must be a thematic break.
```

This input, from `tests/corpus/extensions/frontmatter/not-first-line.etch`, demonstrates that a leading blank line prevents frontmatter recognition. The `---` line is therefore a thematic break, not a frontmatter fence.

`✓ No frontmatter`

```etch
# Direct Heading

This document begins immediately with content and has no frontmatter block.
```

This input, from `tests/corpus/extensions/frontmatter/no-frontmatter.etch`, demonstrates an ordinary document that begins with content rather than frontmatter.
