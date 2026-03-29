use criterion::{Criterion, black_box, criterion_group, criterion_main};
use etch_core::{parse, render_html};

const SMALL_DOC: &str = "\
# Hello World

This is a simple paragraph with **bold** and *italic* text.
";

const MEDIUM_DOC: &str = "\
# Introduction

This is a **bold** and *italic* paragraph with `inline code` and a [link](https://example.com).

## Lists

- First item
- Second item with **bold**
- Third item with a [link](https://example.com)

1. Ordered one
2. Ordered two
3. Ordered three

## Code

```rust
fn main() {
    println!(\"Hello, world!\");
}
```

## Table

| Name  | Value | Description       |
| ----- | ----: | :---------------: |
| Alpha |     1 | The first entry   |
| Beta  |     2 | The second entry  |
| Gamma |     3 | The third entry   |

> A blockquote with *emphasis* and **strong** text.
>
> Second paragraph in the blockquote.

---

Term One
: Definition of term one with *formatting*.

Term Two
: First definition.
: Second definition with **bold**.

Final paragraph with ~~strikethrough~~, ^superscript^, ~subscript~, ==highlight==, and ++insert++ text.
";

fn build_large_doc() -> String {
    MEDIUM_DOC.repeat(20)
}

fn bench_parse(c: &mut Criterion) {
    let large_doc = build_large_doc();

    c.bench_function("parse_small", |b| {
        b.iter(|| parse(black_box(SMALL_DOC)));
    });
    c.bench_function("parse_medium", |b| {
        b.iter(|| parse(black_box(MEDIUM_DOC)));
    });
    c.bench_function("parse_large", |b| {
        b.iter(|| parse(black_box(&large_doc)));
    });
}

fn bench_render(c: &mut Criterion) {
    let small = parse(SMALL_DOC).document;
    let medium = parse(MEDIUM_DOC).document;
    let large_doc = build_large_doc();
    let large = parse(&large_doc).document;

    c.bench_function("render_small", |b| {
        b.iter(|| render_html(black_box(&small)));
    });
    c.bench_function("render_medium", |b| {
        b.iter(|| render_html(black_box(&medium)));
    });
    c.bench_function("render_large", |b| {
        b.iter(|| render_html(black_box(&large)));
    });
}

fn bench_pipeline(c: &mut Criterion) {
    c.bench_function("parse_and_render_medium", |b| {
        b.iter(|| {
            let doc = parse(black_box(MEDIUM_DOC)).document;
            render_html(black_box(&doc))
        });
    });
}

criterion_group!(benches, bench_parse, bench_render, bench_pipeline);
criterion_main!(benches);
