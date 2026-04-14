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
});
