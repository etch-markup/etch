import * as assert from 'node:assert';
import * as vscode from 'vscode';

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
