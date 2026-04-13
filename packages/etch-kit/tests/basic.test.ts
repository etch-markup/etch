import { beforeAll, describe, expect, test } from 'vitest';

import {
  initialize,
  parse,
  parseWithErrors,
  renderDocument,
  renderHtml,
  renderStandalone,
} from '../src/index.js';

describe('etch-kit', () => {
  beforeAll(async () => {
    await initialize();
  });

  test('initialize() completes without error', async () => {
    await expect(initialize()).resolves.toBeUndefined();
  });

  test('initialize() accepts an explicit wasm url', async () => {
    const wasmUrl = new URL('../../crates/etch-wasm/pkg/etch_wasm_bg.wasm', import.meta.url);
    await expect(initialize({ wasmUrl })).resolves.toBeUndefined();
  });

  test('parse() returns a document with one heading', () => {
    const document = parse('# Hello');

    expect(document.body).toHaveLength(1);
    expect(document.body[0]).toMatchObject({
      type: 'Heading',
      level: 1,
    });
  });

  test('renderHtml() returns a heading fragment', () => {
    expect(renderHtml('# Hello')).toContain('<h1 id="hello">Hello</h1>');
  });

  test('renderDocument() returns a full HTML document', () => {
    expect(renderDocument('# Hello').startsWith('<!DOCTYPE html>')).toBe(true);
  });

  test('parseWithErrors() reports mismatched named closes', () => {
    const result = parseWithErrors(':::foo\n:::/bar');

    expect(result.errors).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: 'Error',
          message: expect.stringContaining('expected :::/foo, got :::/bar'),
        }),
      ]),
    );
  });

  test('renderStandalone() returns a full HTML document', () => {
    expect(renderStandalone('# Hello').startsWith('<!DOCTYPE html>')).toBe(true);
  });
});
