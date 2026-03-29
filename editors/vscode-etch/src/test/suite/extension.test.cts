import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { pathToFileURL } from 'node:url';
import * as assert from 'node:assert';
import * as vscode from 'vscode';
import { parseWithErrors, renderDocument } from '../../vendor/etch-kit/index.js';
import { PluginManager } from '../../plugins.js';

suite('Etch Extension', () => {
  const extensionId = 'etch-markup.etch-language';

  suiteSetup(async () => {
    const extension = vscode.extensions.getExtension(extensionId);
    assert.ok(extension, `Extension ${extensionId} should be available.`);
    await extension.activate();
  });

  teardown(async () => {
    await vscode.commands.executeCommand('workbench.action.closeAllEditors');
  });

  test('registers preview commands', async () => {
    const commands = await vscode.commands.getCommands(true);

    assert.ok(commands.includes('etch.openPreview'));
    assert.ok(commands.includes('etch.openPreviewToSide'));
  });

  test('loads the vendored plugin pipeline bundle', async () => {
    const bundlePath = path.resolve(
      __dirname,
      '..',
      '..',
      'vendor',
      'etch-plugin-pipeline',
      'index.js'
    );
    const module = (await import(pathToFileURL(bundlePath).href)) as Record<
      string,
      unknown
    >;

    assert.strictEqual(typeof module.loadConfig, 'function');
    assert.strictEqual(typeof module.createPipeline, 'function');
    assert.strictEqual(typeof module.runPipeline, 'function');
    assert.strictEqual(typeof module.BUILTIN_THEMES, 'object');
  });

  test('reports parse diagnostics for invalid Etch input', async () => {
    const document = await vscode.workspace.openTextDocument({
      language: 'etch',
      content: ':::demo\ncontent\n:::/other\n',
    });

    await vscode.window.showTextDocument(document);

    const diagnostics = await waitForDiagnostics(document.uri, 1);

    assert.ok(
      diagnostics.some(
        (diagnostic) =>
          diagnostic.severity === vscode.DiagnosticSeverity.Error &&
          diagnostic.message.length > 0
      ),
      'Expected at least one error diagnostic for mismatched directive closes.'
    );
  });

  test('does not report diagnostics for valid Etch input', async () => {
    const document = await vscode.workspace.openTextDocument({
      language: 'etch',
      content: '# Title\n\nParagraph text.\n',
    });

    await vscode.window.showTextDocument(document);
    await waitForSettledDiagnostics();

    assert.deepStrictEqual(vscode.languages.getDiagnostics(document.uri), []);
  });

  test('renders math through the core bridge and replaces plugin directives', async () => {
    const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), 'etch-extension-'));

    try {
      writeFileSync(
        path.join(workspaceRoot, 'etch.config.json'),
        JSON.stringify(
          {
            plugins: ['cards'],
            theme: 'academic',
          },
          null,
          2
        )
      );

      const pluginRoot = path.join(
        workspaceRoot,
        '.etch',
        'plugins',
        'etch-plugin-cards'
      );
      mkdirSync(pluginRoot, { recursive: true });
      writeFileSync(
        path.join(pluginRoot, 'package.json'),
        JSON.stringify(
          {
            name: 'etch-plugin-cards',
            version: '1.0.0',
            type: 'module',
            main: 'index.js',
          },
          null,
          2
        )
      );
      writeFileSync(
        path.join(pluginRoot, 'index.js'),
        `export default {
          name: 'cards',
          version: '1.0.0',
          directives: {
            card: {
              type: 'block',
              styles: '.card{color:var(--etch-accent);}',
              render(node, ctx) {
                return '<div class="card">' + node.content + '|' + ctx.renderChildren(node.children) + '</div>';
              }
            }
          }
        };`
      );

      const pluginManager = new PluginManager();
      await pluginManager.initialize(workspaceRoot);

      const source = 'Inline math :math[\\frac{1}{2}].\n\n::card\nhello\n::';
      const parsed = parseWithErrors(source);
      assert.deepStrictEqual(parsed.errors, []);

      const html = renderDocument(source);
      assert.match(
        html,
        /<math xmlns="http:\/\/www\.w3\.org\/1998\/Math\/MathML"><mfrac><mn>1<\/mn><mn>2<\/mn><\/mfrac><\/math>/
      );

      const processed = await pluginManager.processHtml(html, parsed.document);

      assert.match(processed, /<div class="card">hello\|<p>hello<\/p><\/div>/);
      assert.match(processed, /<style data-etch-pipeline="theme">/);
      assert.match(processed, /--etch-bg: #fcfcfa;/);
      assert.match(
        processed,
        /<math xmlns="http:\/\/www\.w3\.org\/1998\/Math\/MathML"><mfrac><mn>1<\/mn><mn>2<\/mn><\/mfrac><\/math>/
      );
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }
  });

  test('falls back when a plugin is missing', async () => {
    const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), 'etch-extension-'));

    try {
      writeFileSync(
        path.join(workspaceRoot, 'etch.config.json'),
        JSON.stringify({ plugins: [], theme: 'default' }, null, 2)
      );

      const pluginManager = new PluginManager();
      await pluginManager.initialize(workspaceRoot);

      const source = 'A :ghost[missing] directive.';
      const parsed = parseWithErrors(source);
      const processed = await pluginManager.processHtml(renderDocument(source), parsed.document);

      assert.match(processed, /class="etch-missing-plugin"/);
      assert.match(processed, /Unknown directive: <code>ghost<\/code>/);
      assert.match(processed, /<style data-etch-pipeline="theme">/);
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }
  });
});

async function waitForDiagnostics(
  uri: vscode.Uri,
  minimumCount: number
): Promise<readonly vscode.Diagnostic[]> {
  const timeoutAt = Date.now() + 5000;

  while (Date.now() < timeoutAt) {
    const diagnostics = vscode.languages.getDiagnostics(uri);

    if (diagnostics.length >= minimumCount) {
      return diagnostics;
    }

    await delay(100);
  }

  return vscode.languages.getDiagnostics(uri);
}

async function waitForSettledDiagnostics(): Promise<void> {
  await delay(300);
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
