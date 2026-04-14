import { beforeAll, describe, expect, test } from 'vitest';

import { initialize, parse } from '../src/index.js';

describe('etch-kit AST surface', () => {
  beforeAll(async () => {
    await initialize();
  });

  test('parses spoiler inline nodes exposed by the Rust AST', () => {
    const document = parse('The answer is ||forty-two||.');
    const paragraph = document.body[0];

    expect(paragraph).toMatchObject({
      type: 'Paragraph',
      content: [
        { type: 'Text', value: 'The answer is ' },
        {
          type: 'Spoiler',
          content: [{ type: 'Text', value: 'forty-two' }],
        },
        { type: 'Text', value: '.' },
      ],
    });
  });

  test('exposes frontmatter objects and attribute pairs as plain JS objects', () => {
    const document = parse(`---
count: 2
nested:
  key: value
tags: [alpha]
---

:card[hello]{tone=warm}`);

    expect(document.frontmatter?.fields).toEqual({
      count: 2,
      nested: {
        key: 'value',
      },
      tags: ['alpha'],
    });
    expect(document.frontmatter?.fields).not.toBeInstanceOf(Map);
    expect(document.frontmatter?.fields.nested).not.toBeInstanceOf(Map);

    const directive = (document.body[0] as { content: Array<{ attrs?: unknown }> }).content[0] as {
      attrs?: { pairs?: Record<string, string> };
    };
    expect(directive.attrs?.pairs).toEqual({ tone: 'warm' });
    expect(directive.attrs?.pairs).not.toBeInstanceOf(Map);
  });

  test('preserves directive fidelity fields used by editor and plugin consumers', () => {
    const document = parse(`:::chapter[Part One]{title=Alpha}
Paragraph
:::/chapter`);

    const directive = document.body[0];

    expect(directive).toMatchObject({
      type: 'ContainerDirective',
      directive_id: expect.any(Number),
      name: 'chapter',
      raw_label: 'Part One',
      raw_body: expect.stringContaining('Paragraph'),
      named_close: true,
      attrs: {
        pairs: {
          title: 'Alpha',
        },
      },
      body: [
        {
          type: 'Paragraph',
        },
      ],
    });
  });
});
