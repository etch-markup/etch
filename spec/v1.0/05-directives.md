# 7. Directives

This chapter defines Etch directives. Directives are the language's uniform extension mechanism and appear in three surface forms: inline directives, block directives, and container directives.

Directive parsing is normative. A conforming parser MUST apply the recognition, content, closing, and nesting rules in this chapter exactly as written.

## 7.1 Directive Anatomy

### Structure

1. **Directive marker**: A directive begins with one of the following markers:
   1. `:` for an inline directive;
   2. `::` for a block directive; or
   3. `:::` for a container directive.
2. **Name**: Every directive has a name. A directive name MUST begin with a letter and MUST contain only letters (`a-zA-Z`) and hyphens (`-`). Digits and underscores are not permitted in directive names.
3. **Optional bracketed content**: A directive MAY carry bracketed content or label text in `[...]`. When present, the bracketed region closes only at the `]` that returns bracket depth to zero. Nested balanced brackets therefore work automatically.
4. **Escaping inside bracketed content**: When literal unbalanced `]` is needed inside bracketed content, it MUST be written as `\]`.
5. **Optional attributes**: A directive MAY carry an attribute set in `{...}`. Directive attributes reuse the same syntax and escaping rules as ordinary element attributes.
6. **Ordering**: When both bracketed content and attributes are present, the bracketed content MUST appear before the attribute set: `:name[text]{key=value}`, `::name[label]{key=value}`, `:::name[label]{key=value}`.
7. **Inline parsing of bracketed content**: For inline directives, and for the optional label on block and container directives, bracketed content is parsed recursively as inline content after the balanced-bracket scan identifies its extent.

### Normative Examples

`✓ Valid names`

```etch
:aside
:character-card[Sable]
:content-warning[level one]
```

These examples use names consisting only of letters and hyphens.

`✗ Invalid names`

```etch
:widget_v2[text]
:col3[text]
:_internal[text]
```

These examples are invalid because directive names MUST NOT contain underscores or digits and MUST begin with a letter.

`✓ Balanced bracket content`

```etch
:tooltip[array[0] is the first element]
:code[fn(a, b) -> Result<Vec<[u8]>>]
```

This input, from `tests/corpus/extensions/inline-directives/balanced-brackets.etch`, demonstrates that balanced nested brackets do not terminate the content early.

`✓ Escaped closing bracket`

```etch
:tooltip[use \] here]
```

This input, from `tests/corpus/extensions/inline-directives/escaped-bracket.etch`, demonstrates the required escape form for an otherwise unbalanced closing bracket.

## 7.2 Inline Directives

### Structure

1. **Recognition rule**: An inline directive begins only when `:` is immediately followed by a letter (`[a-zA-Z]`). If the character after `:` is a space, digit, slash, or any other non-letter character, the `:` remains plain text and no directive begins.
2. **Bare form**: The bare form `:name` is valid. An inline directive therefore does not require bracketed content or attributes.
3. **Optional content**: An inline directive MAY include bracketed content immediately after the name: `:name[...]`.
4. **Optional attributes**: An inline directive MAY include an attribute set immediately after the name or immediately after bracketed content: `:name{...}` or `:name[...]{...}`.
5. **Content model**: When bracketed content is present, that content is parsed recursively as inline content. It MAY therefore contain text, emphasis, strong, links, autolinks, code spans, footnote references, and the other inline forms defined in Chapter 5.
6. **Termination rule**: The directive ends after its name, optional bracketed content, and optional attribute set. Parsing of surrounding inline content then resumes immediately after the directive.
7. **Disambiguation**: `: `, `:0`, `:/`, and similar non-letter continuations are not directives. Examples such as `note: text`, `3:00pm`, and `https://example.com` therefore remain ordinary text or other inline constructs.

### Normative Examples

`✓ Bare`

```etch
Insert a :pagebreak here.
```

This input, from `tests/corpus/extensions/inline-directives/bare.etch`, demonstrates that `:pagebreak` is a valid inline directive even though it has no bracketed content and no attributes.

`✓ With content and attributes`

```etch
I met :character[Sable]{species=fox} at the cafe.
Today is :date{format=long}.
```

These forms are exercised by `tests/corpus/extensions/inline-directives/with-both.etch` and `tests/corpus/extensions/inline-directives/with-attrs.etch`.

`✓ Recursive inline parsing inside content`

```etch
The margin note says :tooltip[**bold** and *italic* content]{text="extra info"} beside the chart.
```

This input, from `tests/corpus/extensions/inline-directives/content-with-formatting.etch`, demonstrates that inline formatting inside bracketed directive content is recursively parsed.

`✗ Not a directive`

```etch
Dear friend: I have news from the harbor.
The time is 3:00pm when the bell rings.
See https://example.com for the route map.
Note: this is important before you leave.
```

This input, from `tests/corpus/extensions/inline-directives/not-a-directive.etch`, demonstrates that a colon followed by a space, digit, or slash does not begin a directive.

## 7.3 Block Directives

### Structure

1. **Recognition rule**: A block directive opens on a line whose first two characters are `::`, immediately followed by a valid directive name. The opening line MAY also contain an optional label in `[...]` and an optional attribute set in `{...}`.
2. **Opening-line form**: The complete opening line is one of `::name`, `::name[...]`, `::name{...}`, or `::name[...]{...}`.
3. **Body**: The body of a block directive consists of every subsequent source line until the next later line that is exactly `::`.
4. **Fenced behavior**: Blank lines inside a block directive body do not terminate the directive. Only the closing fence line `::` terminates it.
5. **Content model**: The body of a block directive is parsed recursively as block content.
6. **Leaf semantics**: A block directive is a leaf directive. Its body MAY contain paragraphs, lists, headings, blockquotes, tables, and other ordinary block content, but it MUST NOT contain nested block directives or nested container directives.
7. **Inline directives inside leaf content**: Inline directives remain valid inside the textual content of descendant blocks within a leaf directive, because they are part of ordinary recursive inline parsing rather than nested directive blocks.
8. **Termination rule**: The first later line that is exactly `::` closes the block directive. If no such line appears, the block directive consumes the remainder of the input.
9. **Empty body**: A block directive MAY have an empty body.

### Normative Examples

`✓ Basic fenced block`

```etch
::aside
First paragraph of the aside.
::
```

This input, from `tests/corpus/extensions/block-directives/basic.etch`, produces one `BlockDirective` named `aside`.

`✓ With label and attributes`

```etch
::aside[Author's note]{.highlighted}
This margin note should appear with highlighted styling.
::
```

This input, from `tests/corpus/extensions/block-directives/with-both.etch`, demonstrates the full opening-line form for a block directive.

`✓ Fenced behavior with blank lines`

```etch
::aside
First paragraph in the aside.



Second paragraph still belongs to the same directive.

Third paragraph also remains inside until the closing fence.
::
```

This input, from `tests/corpus/extensions/block-directives/blank-lines-inside.etch`, demonstrates that blank lines do not end a block directive body.

`✓ Rich block content`

```etch
::aside
First paragraph inside the aside.

Second paragraph still belongs to the same directive.

- Pack the spare map
- Bring the brass compass
- Check the tide chart

The third paragraph ends with **bold** emphasis.
::
```

This input, from `tests/corpus/extensions/block-directives/rich-content.etch`, demonstrates recursive block parsing inside a block directive body.

`✓ Inline directives inside a leaf`

```etch
::aside
The margin note uses :tooltip[text]{info="x"} beside :math[E=mc^2] in the same paragraph.
::
```

This input, from `tests/corpus/extensions/nesting/inline-in-leaf.etch`, is valid because inline directives remain part of text flow inside the leaf directive's body.

`✗ Nested directive inside a leaf`

```etch
::aside
This leaf directive starts with valid text.

::spoiler
This nested directive should cause a parse error.
::
::
```

This input, from `tests/corpus/extensions/nesting/leaf-rejects-directive.etch`, is invalid because a leaf directive body MUST NOT contain nested block directives or container directives.

## 7.4 Container Directives

### Structure

1. **Recognition rule**: A container directive opens on a line whose first three characters are `:::`, immediately followed by a valid directive name. The opening line MAY also contain an optional label in `[...]` and an optional attribute set in `{...}`.
2. **Opening-line form**: The complete opening line is one of `:::name`, `:::name[...]`, `:::name{...}`, or `:::name[...]{...}`.
3. **Content model**: The body of a container directive is parsed recursively as block content.
4. **Structural semantics**: A container directive is structural by default. Its body MAY contain ordinary blocks, block directives, and other container directives.
5. **Anonymous close**: A container directive MAY close with a line that is exactly `:::`.
6. **Named close**: A container directive MAY instead close with a line of the form `:::/name`.
7. **Matching rule**: When a named close is used, the closing name MUST exactly match the opening directive name. A mismatched named close is an error.
8. **Termination rule**: The body extends until the matching anonymous or named close line is reached. If no valid close is found, the container directive consumes the remainder of the input and the missing close is an error condition.

### Normative Examples

`✓ Anonymous close`

```etch
:::chapter{title="One"}
The first chapter opens with a short paragraph.
:::
```

This input, from `tests/corpus/extensions/container-directives/anonymous-close.etch`, demonstrates the anonymous closing form.

`✓ Named close`

```etch
:::chapter{title="One"}
The named close should match the opening directive.
:::/chapter
```

This input, from `tests/corpus/extensions/container-directives/named-close.etch`, demonstrates the named closing form.

`✓ Nested directives inside a container`

```etch
:::chapter{title="Storm Journal"}
::aside
Keep the lamp away from the wet ledge.
::

::spoiler[Hidden note]
The spare key is under the blue crate.
::
:::/chapter
```

This input, from `tests/corpus/extensions/container-directives/with-block-directives.etch`, demonstrates that a structural container may contain block directives.

`✓ Canonical structural nesting`

```etch
:::columns{count=2 gap="1rem"}
:::column
## Left Notes

Pack the stove fuel and the spare wick.
:::/column
:::column
## Right Notes

Leave the tide chart with the dockmaster.
:::/column
:::/columns
```

This input, from `tests/corpus/extensions/container-directives/columns-pattern.etch`, demonstrates nested structural containers using named closes.

`✗ Mismatched named close`

```etch
:::chapter{title="One"}
This container is closed with the wrong directive name.
:::/section
```

This input, from `tests/corpus/extensions/container-directives/mismatched-close.etch`, is invalid because the named close does not match the opening name.

## 7.5 Nesting and Directive Types

### Structure

1. **Directive type system**: For nesting validation, directives are classified as either `leaf` or `structural`.
2. **Leaf directives**: A leaf directive MAY contain ordinary block content and inline content, but it MUST NOT contain nested block directives or nested container directives.
3. **Structural directives**: A structural directive MAY contain ordinary block content, block directives, and other structural directives.
4. **Inline directives in leaf bodies**: Inline directives that appear within paragraph text, heading text, list-item text, or other inline-bearing content inside a leaf directive are valid. They do not count as nested block-level directives.
5. **Defaults**: Unknown or unregistered directive names MUST default to `leaf` unless an implementation explicitly declares them structural.
6. **Structural nesting warning**: Structural nesting beyond six levels is valid but SHOULD produce a linter warning. It is not a parse error.

### Normative Examples

`✓ Structural in structural`

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

This pattern, exercised by `tests/corpus/extensions/nesting/structural-in-structural.etch` and `tests/corpus/extensions/nesting/depth-4-warning.etch`, is valid structural nesting.

`✓ Valid but warns at depth six`

```etch
:::chapter{title="One"}
:::section{title="Two"}
:::columns{count=2}
:::column
:::stack
:::pane
Deep content lives here.
:::/pane
:::/stack
:::/column
:::/columns
:::/section
:::/chapter
```

This input, from `tests/corpus/extensions/nesting/depth-4-warning.etch`, is valid but SHOULD trigger a linter warning because it reaches six structural levels.

`✗ Leaf contains directive`

```etch
::aside
This leaf directive starts with valid text.

::spoiler
This nested directive should cause a parse error.
::
::
```

This input is invalid for the reason described in Section 7.3: a leaf directive cannot contain nested block or container directives.

## 7.6 Disambiguation Summary

The following directive disambiguation rules are normative:

1. `:` begins an inline directive only when the next character is a letter.
2. `::` begins a block directive only when the next character is a letter.
3. `:::` begins a container directive only when the next character is a letter.
4. `: `, `:0`, `:/`, and similar non-letter continuations remain literal text.
5. `::` and `:::` used as closing fences are recognized only in block and container parsing contexts, on lines by themselves, as defined above.

`✓ Summary example`

```etch
Dear friend: I have news from the harbor.
The time is 3:00pm when the bell rings.
See https://example.com for the route map.
The margin note uses :tooltip[text]{info="x"} in the same paragraph.
```

This combined example demonstrates plain-text colons alongside one valid inline directive.
