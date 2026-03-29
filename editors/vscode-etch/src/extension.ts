import * as vscode from 'vscode';
import { initialize } from './vendor/etch-kit/index.js';
import { EtchPreviewManager } from './preview.js';

const ETCH_LANGUAGE_ID = 'etch';

type PreviewPluginManager = {
  initialize(workspaceRoot: string): Promise<void>;
  processHtml(html: string, document: unknown): Promise<string>;
  watchChanges(onReload: () => void): vscode.Disposable;
  dispose(): void;
};

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const diagnostics = vscode.languages.createDiagnosticCollection(ETCH_LANGUAGE_ID);
  context.subscriptions.push(diagnostics);

  await initialize();

  const pluginManager = await createPluginManager();
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (workspaceRoot) {
    await pluginManager.initialize(workspaceRoot);
  }

  const previewManager = new EtchPreviewManager(context, diagnostics, pluginManager);

  context.subscriptions.push(
    pluginManager,
    previewManager,
    workspaceRoot
      ? pluginManager.watchChanges(() => {
          void previewManager.refreshAllPreviews();
        })
      : new vscode.Disposable(() => undefined),
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
    }),
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      previewManager.handleActiveEditorChange(editor);
    })
  );

  for (const document of vscode.workspace.textDocuments) {
    previewManager.refreshDocument(document);
  }
}

export function deactivate(): void {}

async function createPluginManager(): Promise<PreviewPluginManager> {
  try {
    const { PluginManager } = await import('./plugins.js');
    return new PluginManager();
  } catch (error) {
    console.error('[etch] Failed to load plugin manager; continuing without plugin pipeline.', error);
    return new NoopPluginManager();
  }
}

class NoopPluginManager implements PreviewPluginManager {
  public async initialize(_workspaceRoot: string): Promise<void> {}

  public async processHtml(html: string, _document: unknown): Promise<string> {
    return html;
  }

  public watchChanges(_onReload: () => void): vscode.Disposable {
    return new vscode.Disposable(() => undefined);
  }

  public dispose(): void {}
}
