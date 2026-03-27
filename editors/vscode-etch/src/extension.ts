import * as vscode from 'vscode';
import { initialize } from './etch-kit/index.js';
import { EtchPreviewManager } from './preview.js';

const ETCH_LANGUAGE_ID = 'etch';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const diagnostics = vscode.languages.createDiagnosticCollection(ETCH_LANGUAGE_ID);
  context.subscriptions.push(diagnostics);

  await initialize();

  const previewManager = new EtchPreviewManager(context, diagnostics);

  context.subscriptions.push(
    previewManager,
    vscode.commands.registerCommand('etch.openPreview', async () => {
      await previewManager.openPreview(false);
    }),
    vscode.commands.registerCommand('etch.openPreviewToSide', async () => {
      await previewManager.openPreview(true);
    }),
    vscode.workspace.onDidOpenTextDocument((document) => {
      previewManager.refreshDocument(document);
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      previewManager.handleDocumentChange(event.document);
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      previewManager.handleDocumentClose(document);
    })
  );

  for (const document of vscode.workspace.textDocuments) {
    previewManager.refreshDocument(document);
  }
}

export function deactivate(): void {}
