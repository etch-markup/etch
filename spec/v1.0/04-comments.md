# 6. Comments

This chapter defines Etch comments. Comments are source-only annotations and do not contribute block elements, inline elements, or text nodes to the document model.

## 6.1 Comment Syntax

### Structure

1. **Opening delimiter**: A comment opens with the two-character sequence `{~`.
2. **Closing delimiter**: A comment closes with the next later occurrence of the two-character sequence `~}`.
3. **Forms**: A comment MAY appear inline within surrounding text, MAY occupy an entire source line, or MAY span multiple lines.
4. **Content model**: Comment content is raw source text. Block parsing, inline parsing, directive parsing, and attribute parsing MUST NOT occur inside an open comment.
5. **Stripping behavior**: Except inside the body of a fenced code block, a conforming parser MUST remove comments from the source before block parsing and inline parsing. An implementation MAY realize this as a preprocessing pass or by any equivalent mechanism, provided the resulting document model is the same as if each `{~ ... ~}` span had been removed first.
6. **No nesting**: Comment nesting is not supported. A `{~` sequence that appears inside an already open comment is literal comment content and MUST NOT begin a nested comment. The first `~}` after the opening delimiter MUST close the comment.
7. **Fenced code interaction**: The body of a fenced code block is raw text. A `{~` sequence inside a fenced code block body MUST NOT begin a comment, and a `~}` sequence inside a fenced code block body MUST NOT close one.
8. **Attribute interaction**: When comment recognition is active, `{~` begins a comment. Attribute parsing uses distinct openings such as `{#` and `{.` and MUST NOT be confused with `{~`. A comment and a following attribute set MAY appear adjacent in source and remain separate constructs.
9. **Subscript interaction**: The comment closer is the exact two-character sequence `~}` recognized during comment stripping. Subscript uses single `~` delimiters recognized later during inline parsing. Therefore `~sub~` MUST NOT be confused with comment syntax.

### Normative Examples

`✓ Inline`

```etch
Text before {~ this is hidden ~} text after.
```

This input, from `tests/corpus/extensions/comments/inline.etch`, produces one `Paragraph`. The comment contributes no inline nodes and is removed before inline parsing proceeds on the surrounding text.

`✓ Full-line`

```etch
The first paragraph appears before the comment.

{~ This entire line is a comment ~}

The second paragraph appears after the comment.
```

This input, from `tests/corpus/extensions/comments/line.etch`, produces two `Paragraph` blocks. The full-line comment contributes no block of its own.

`✓ Multi-line`

```etch
Paragraph before the comment block.

{~
This comment spans multiple lines.
It holds author notes and reminders.
The parser should ignore all of it.
~}

Paragraph after the comment block.
```

This input, from `tests/corpus/extensions/comments/multi-line.etch`, produces two `Paragraph` blocks. The multi-line comment contributes no block elements, inline elements, or text nodes.

`✓ No nesting`

```etch
Before {~ outer comment {~ this looks like inner ~} but the comment closed at the first ~} and this is visible text ~}.
```

This input, from `tests/corpus/extensions/comments/no-nesting.etch`, closes the comment at the first `~}`. The inner-looking `{~` is literal text inside the comment, and the later `~}` sequences remain visible literal text after comment stripping.

`✓ Comment inside a heading`

```etch
# My Title {~ draft version ~}
```

This input, from `tests/corpus/extensions/comments/in-heading.etch`, produces one level-1 `Heading`. The comment is removed before the heading's inline content is parsed.

`✓ Adjacent to attributes`

```etch
# Title {~ note to self ~} {#my-id}
```

This input, from `tests/corpus/extensions/comments/adjacent-to-attrs.etch`, produces one level-1 `Heading` with identifier `my-id`. The `{~ ... ~}` span is recognized as a comment, while the following `{#my-id}` is still recognized as the heading's attribute set.

`✓ Not recognized inside fenced code`

````etch
```
status = "**draft**"
:note[remember this]
{~ hidden in code ~}
```
````

This input is governed by the fenced-code rules in Chapter 4 and produces one `CodeBlock`. The `{~ hidden in code ~}` line remains literal code text and is not removed as a comment.

`✓ Distinct from subscript`

```etch
Water is H~2~O in the field notes {~ chemistry reminder ~} and the stove emits CO~2~.
```

This input, from `tests/corpus/extensions/comments/nested-tilde.etch`, produces `Subscript` nodes for the two `~2~` spans and removes the `{~ chemistry reminder ~}` comment. The single-tilde subscript delimiters are not confused with the `~}` comment closer.
