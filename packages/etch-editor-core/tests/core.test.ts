import { afterEach, describe, expect, test, vi } from 'vitest';

import { createEtchEditorCore } from '../src/index.js';

describe('etch-editor-core', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  test('initializes and renders standalone preview html', async () => {
    const core = createEtchEditorCore({ initialSource: '# Hello' });

    try {
      const state = await waitForState(
        core,
        (current) => !current.isInitializing && current.previewHtml.includes('<h1 id="hello">Hello</h1>')
      );

      expect(state.errors).toEqual([]);
      expect(state.previewHtml).toContain('<!DOCTYPE html>');
      expect(state.previewHtml).toContain('<main>');
    } finally {
      core.dispose();
    }
  });

  test('debounces source updates', async () => {
    vi.useFakeTimers();

    const core = createEtchEditorCore({ initialSource: '# One' });

    try {
      await waitForState(core, (state) => !state.isInitializing && state.previewHtml.includes('One'));

      core.setSource('# Two');

      expect(core.getState().source).toBe('# Two');
      expect(core.getState().previewHtml).toContain('One');

      await vi.advanceTimersByTimeAsync(199);
      expect(core.getState().previewHtml).toContain('One');

      await vi.advanceTimersByTimeAsync(1);
      expect(core.getState().previewHtml).toContain('Two');
    } finally {
      core.dispose();
    }
  });

  test('updates preview styles when the theme changes', async () => {
    const core = createEtchEditorCore({ initialSource: '# Theme' });

    try {
      await waitForState(core, (state) => !state.isInitializing && state.previewHtml.length > 0);

      core.setTheme('academic');

      const state = await waitForState(
        core,
        (current) => current.theme === 'academic' && current.previewHtml.includes('--etch-bg: #fcfcfa;')
      );

      expect(state.previewHtml).toContain('@media (prefers-color-scheme: dark)');
    } finally {
      core.dispose();
    }
  });

  test('surfaces parse diagnostics while preserving preview output', async () => {
    const core = createEtchEditorCore({ initialSource: ':::foo\n:::/bar' });

    try {
      const state = await waitForState(
        core,
        (current) => !current.isInitializing && current.errors.length > 0
      );

      expect(state.errors[0]?.message).toContain('expected :::/foo, got :::/bar');
      expect(state.previewHtml).toContain('<!DOCTYPE html>');
    } finally {
      core.dispose();
    }
  });
});

function waitForState(
  core: ReturnType<typeof createEtchEditorCore>,
  predicate: (state: ReturnType<typeof core.getState>) => boolean,
  timeoutMs = 3000
): Promise<ReturnType<typeof core.getState>> {
  const initialState = core.getState();
  if (predicate(initialState)) {
    return Promise.resolve(initialState);
  }

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      unsubscribe();
      reject(new Error('Timed out waiting for editor state.'));
    }, timeoutMs);

    const unsubscribe = core.subscribe((state) => {
      if (!predicate(state)) {
        return;
      }

      clearTimeout(timeout);
      unsubscribe();
      resolve(state);
    });
  });
}
