import * as assert from 'node:assert';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';

suite('Etch Extension Sync', () => {
  test('keeps shared editor assets mirrored into the VS Code extension', async () => {
    const sharedGrammar = await readRepoFile('editors', 'shared', 'etch.tmLanguage.json');
    const vscodeGrammar = await readRepoFile(
      'editors',
      'vscode-etch',
      'syntaxes',
      'etch.tmLanguage.json'
    );
    const sharedLanguageConfig = await readRepoFile(
      'editors',
      'shared',
      'language-configuration.json'
    );
    const vscodeLanguageConfig = await readRepoFile(
      'editors',
      'vscode-etch',
      'language-configuration.json'
    );

    assert.strictEqual(
      vscodeGrammar,
      sharedGrammar,
      'Expected editors/shared/etch.tmLanguage.json and the VS Code copy to stay byte-identical.'
    );
    assert.strictEqual(
      vscodeLanguageConfig,
      sharedLanguageConfig,
      'Expected editors/shared/language-configuration.json and the VS Code copy to stay byte-identical.'
    );
  });

  test('keeps vendored etch-kit shims pinned to the package build outputs', async () => {
    const typeShim = await readRepoFile(
      'editors',
      'vscode-etch',
      'src',
      'vendor',
      'etch-kit',
      'index.d.ts'
    );
    const runtimeShim = await readRepoFile(
      'editors',
      'vscode-etch',
      'vendor',
      'etch-kit',
      'index.ts'
    );

    assert.strictEqual(
      typeShim.trim(),
      "export * from '../../../../../packages/etch-kit/dist/index.js';",
      'Expected the vendored type shim to re-export the built @etch-markup/etch-kit contract directly.'
    );
    assert.match(
      runtimeShim,
      /from '\.\.\/\.\.\/\.\.\/\.\.\/packages\/etch-kit\/dist\/index\.js';/,
      'Expected the vendored runtime shim to import the built @etch-markup/etch-kit runtime.'
    );
    assert.match(
      runtimeShim,
      /from '\.\.\/\.\.\/\.\.\/\.\.\/packages\/etch-kit\/dist\/runtime\.js';/,
      'Expected the vendored runtime shim to import the built @etch-markup/etch-kit runtime helpers.'
    );
  });
});

async function readRepoFile(...segments: string[]): Promise<string> {
  const filePath = path.resolve(__dirname, '..', '..', '..', '..', '..', ...segments);
  return fs.readFile(filePath, 'utf8');
}
