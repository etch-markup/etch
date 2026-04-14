import { describe, expect, test } from 'vitest';

import { resolveEtchEditorCoreOptions } from '../src/hooks/useEtchEditor.js';

describe('useEtchEditor contract', () => {
  test('treats value as the authoritative initial source when controlled', () => {
    expect(
      resolveEtchEditorCoreOptions({
        initialSource: '# Draft',
        value: '# Controlled',
      })
    ).toMatchObject({
      initialSource: '# Controlled',
    });
  });

  test('uses initialSource for uncontrolled mounts', () => {
    expect(
      resolveEtchEditorCoreOptions({
        initialSource: '# Uncontrolled',
      })
    ).toMatchObject({
      initialSource: '# Uncontrolled',
    });
  });

  test('preserves theme and wasm initialization options', () => {
    expect(
      resolveEtchEditorCoreOptions({
        initialSource: '# Hello',
        theme: 'paper',
        wasmUrl: '/etch.wasm',
      })
    ).toMatchObject({
      initialSource: '# Hello',
      theme: 'paper',
      wasmUrl: '/etch.wasm',
    });
  });
});
