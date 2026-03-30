import * as vscode from 'vscode';
import {
  DEFAULT_STANDALONE_STYLES,
  parseWithErrors,
  renderDocument,
  type ParseError,
  type Document as EtchDocument,
} from './vendor/etch-kit/index.js';

const ETCH_LANGUAGE_ID = 'etch';
const PREVIEW_DEBOUNCE_MS = 200;

type PreviewEntry = {
  documentUri: string;
  panel: vscode.WebviewPanel;
  sourceColumn: vscode.ViewColumn;
  previewColumn: vscode.ViewColumn;
};

type PreviewPluginManager = {
  processHtml(html: string, document: EtchDocument): Promise<string>;
};

export class EtchPreviewManager implements vscode.Disposable {
  private readonly context: vscode.ExtensionContext;
  private readonly diagnostics: vscode.DiagnosticCollection;
  private readonly pluginManager: PreviewPluginManager;
  private readonly panelsByDocument = new Map<string, PreviewEntry>();
  private readonly panelsBySourceColumn = new Map<string, PreviewEntry>();
  private readonly refreshTimers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly renderVersions = new Map<string, number>();

  public constructor(
    context: vscode.ExtensionContext,
    diagnostics: vscode.DiagnosticCollection,
    pluginManager: PreviewPluginManager
  ) {
    this.context = context;
    this.diagnostics = diagnostics;
    this.pluginManager = pluginManager;
  }

  public async openPreview(toSide: boolean): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;

    if (document?.languageId !== ETCH_LANGUAGE_ID) {
      vscode.window.showInformationMessage(
        'Open an Etch document to preview it.'
      );
      return;
    }

    const key = document.uri.toString();
    const sourceColumn = this.resolveSourceColumn(editor?.viewColumn);
    const sourceKey = String(sourceColumn);
    const existing = this.panelsBySourceColumn.get(sourceKey);
    const existingForDocument = this.panelsByDocument.get(key);
    const targetColumn = toSide ? this.getSideColumn(sourceColumn) : sourceColumn;

    if (existing) {
      this.bindPreviewToDocument(existing, document, sourceColumn, targetColumn);
      existing.panel.title = this.getPanelTitle(document);
      existing.panel.reveal(targetColumn, true);
      await this.renderPreview(document, existing.panel);
      return;
    }

    if (existingForDocument) {
      this.bindPreviewToDocument(
        existingForDocument,
        document,
        sourceColumn,
        targetColumn
      );
      existingForDocument.panel.title = this.getPanelTitle(document);
      existingForDocument.panel.reveal(targetColumn, true);
      await this.renderPreview(document, existingForDocument.panel);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      'etchPreview',
      this.getPanelTitle(document),
      {
        viewColumn: targetColumn,
        preserveFocus: true,
      },
      {
        enableScripts: false,
        localResourceRoots: [vscode.Uri.joinPath(this.context.extensionUri, 'media')],
      }
    );

    panel.iconPath = this.getPreviewIconPath();

    const entry = {
      documentUri: key,
      panel,
      sourceColumn,
      previewColumn: targetColumn,
    };
    this.registerPreview(entry);

    panel.onDidDispose(() => {
      this.unregisterPreview(entry);
    });

    await this.renderPreview(document, panel);
  }

  public refreshDocument(document: vscode.TextDocument): void {
    if (document.languageId !== ETCH_LANGUAGE_ID) {
      return;
    }

    this.renderDiagnostics(document);
  }

  public handleDocumentChange(document: vscode.TextDocument): void {
    if (document.languageId !== ETCH_LANGUAGE_ID) {
      return;
    }

    const key = document.uri.toString();
    const pending = this.refreshTimers.get(key);

    if (pending) {
      clearTimeout(pending);
    }

    const timer = setTimeout(() => {
      this.refreshTimers.delete(key);
      void this.updateDocument(document);
    }, PREVIEW_DEBOUNCE_MS);

    this.refreshTimers.set(key, timer);
  }

  public handleDocumentClose(document: vscode.TextDocument): void {
    const key = document.uri.toString();
    const pending = this.refreshTimers.get(key);

    if (pending) {
      clearTimeout(pending);
      this.refreshTimers.delete(key);
    }

    this.diagnostics.delete(document.uri);
    this.renderVersions.delete(key);

    // Only remove the document mapping — keep the panel alive and in the
    // source-column map so handleActiveEditorChange can rebind it to the
    // next active etch file (mirrors the Markdown preview behavior).
    this.panelsByDocument.delete(key);
  }

  public handleActiveEditorChange(editor: vscode.TextEditor | undefined): void {
    const document = editor?.document;

    if (document?.languageId !== ETCH_LANGUAGE_ID) {
      return;
    }

    const sourceColumn = this.resolveSourceColumn(editor?.viewColumn);
    const entry = this.panelsBySourceColumn.get(String(sourceColumn));

    if (!entry || entry.documentUri === document.uri.toString()) {
      return;
    }

    this.bindPreviewToDocument(
      entry,
      document,
      sourceColumn,
      entry.previewColumn
    );
    void this.renderPreview(document, entry.panel);
  }

  public dispose(): void {
    for (const timer of this.refreshTimers.values()) {
      clearTimeout(timer);
    }

    this.refreshTimers.clear();
    this.renderVersions.clear();

    for (const entry of this.panelsByDocument.values()) {
      entry.panel.dispose();
    }

    this.panelsByDocument.clear();
    this.panelsBySourceColumn.clear();
  }

  public async refreshAllPreviews(): Promise<void> {
    for (const [key, entry] of this.panelsByDocument.entries()) {
      const document = vscode.workspace.textDocuments.find(
        (candidate) => candidate.uri.toString() === key
      );
      if (!document) {
        continue;
      }
      await this.renderPreview(document, entry.panel);
    }
  }

  private async updateDocument(document: vscode.TextDocument): Promise<void> {
    const entry = this.panelsByDocument.get(document.uri.toString());

    if (!entry) {
      this.renderDiagnostics(document);
      return;
    }

    await this.renderPreview(document, entry.panel);
  }

  private renderDiagnostics(document: vscode.TextDocument): ParseError[] {
    const result = parseWithErrors(document.getText());
    this.setDiagnostics(document, result.errors);
    return result.errors;
  }

  private async renderPreview(
    document: vscode.TextDocument,
    panel: vscode.WebviewPanel
  ): Promise<void> {
    const key = document.uri.toString();
    const renderVersion = (this.renderVersions.get(key) ?? 0) + 1;
    this.renderVersions.set(key, renderVersion);

    try {
      const parseResult = parseWithErrors(document.getText());
      this.setDiagnostics(document, parseResult.errors);
      const coreHtml = renderDocument(document.getText());
      const rendered = await this.pluginManager.processHtml(coreHtml, parseResult.document);

      if (!this.isLatestRender(key, renderVersion, panel)) {
        return;
      }

      panel.title = this.getPanelTitle(document);
      panel.webview.html = this.decorateHtml(
        panel.webview,
        rendered,
        parseResult.errors
      );
    } catch (error) {
      if (!this.isLatestRender(key, renderVersion, panel)) {
        return;
      }
      panel.webview.html = this.renderFailure(panel.webview, error);
    }
  }

  private decorateHtml(
    webview: vscode.Webview,
    html: string,
    errors: ParseError[]
  ): string {
    const previewCssUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, 'media', 'preview.css')
    );

    const headInjection = [
      `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} https: data:; style-src ${webview.cspSource} 'unsafe-inline'; font-src ${webview.cspSource} https: data:;">`,
      `<link rel="stylesheet" href="${previewCssUri.toString()}">`,
      `<style>${DEFAULT_STANDALONE_STYLES}</style>`,
      `<style>
        :root {
          --etch-muted: var(--vscode-descriptionForeground, var(--etch-text));
          --etch-border: var(--vscode-panel-border, rgba(127, 127, 127, 0.2));
          --etch-surface: var(--vscode-editorWidget-background, rgba(127, 127, 127, 0.08));
          --etch-surface-strong: var(--vscode-textBlockQuote-background, rgba(127, 127, 127, 0.12));
          --etch-code-text: var(--vscode-editor-foreground);
          --etch-warning-bg: var(--vscode-inputValidation-warningBackground, rgba(255, 196, 0, 0.12));
          --etch-warning-border: var(--vscode-inputValidation-warningBorder, rgba(255, 196, 0, 0.4));
          --etch-warning-text: var(--vscode-inputValidation-warningForeground, var(--etch-text));
        }
      </style>`,
    ].join('\n');

    const banner = this.renderErrorBanner(errors);
    const withMain = ensureMainWrapper(html);
    const withHead = injectBeforeClosingTag(withMain, 'head', headInjection);
    return injectAfterOpeningBody(withHead, banner);
  }

  private renderFailure(webview: vscode.Webview, error: unknown): string {
    const previewCssUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, 'media', 'preview.css')
    );

    const message = error instanceof Error ? error.message : String(error);

    return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline';">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="${previewCssUri.toString()}">
    <style>
      :root {
        --etch-bg: var(--vscode-editor-background);
        --etch-text: var(--vscode-editor-foreground);
      }
    </style>
    <title>Etch Preview Error</title>
  </head>
  <body>
    <div class="etch-preview-fallback">
      <h1>Preview failed</h1>
      <pre>${escapeHtml(message)}</pre>
    </div>
  </body>
</html>`;
  }

  private renderErrorBanner(errors: ParseError[]): string {
    if (errors.length === 0) {
      return '';
    }

    const highestSeverity = errors.some((error) => error.kind === 'Error')
      ? 'error'
      : 'warning';

    const summary =
      highestSeverity === 'error'
        ? 'Etch parser reported errors.'
        : 'Etch parser reported warnings.';

    const items = errors
      .map((error) => {
        const location = error.column
          ? `Line ${error.line}, column ${error.column}`
          : `Line ${error.line}`;

        return `<li><strong>${escapeHtml(location)}</strong>: ${escapeHtml(
          error.message
        )}</li>`;
      })
      .join('');

    return `<div class="etch-preview-errors ${
      highestSeverity === 'warning' ? 'is-warning' : ''
    }">
  <strong>${summary}</strong>
  <ul>${items}</ul>
</div>`;
  }

  private toDiagnostic(
    document: vscode.TextDocument,
    error: ParseError
  ): vscode.Diagnostic {
    const lineIndex = Math.max(error.line - 1, 0);
    const line = document.lineAt(Math.min(lineIndex, document.lineCount - 1));
    const startColumn = Math.min(
      Math.max((error.column ?? 1) - 1, 0),
      line.range.end.character
    );
    const hasColumn = typeof error.column === 'number';
    const endColumn = hasColumn
      ? Math.min(startColumn + 1, line.range.end.character)
      : line.range.end.character;
    const range = hasColumn
      ? new vscode.Range(
          line.lineNumber,
          startColumn,
          line.lineNumber,
          Math.max(endColumn, startColumn)
        )
      : line.range;

    return new vscode.Diagnostic(
      range,
      error.message,
      error.kind === 'Warning'
        ? vscode.DiagnosticSeverity.Warning
        : vscode.DiagnosticSeverity.Error
    );
  }

  private getPanelTitle(document: vscode.TextDocument): string {
    return `Preview: ${this.getDocumentLabel(document)}`;
  }

  private getSideColumn(
    column: vscode.ViewColumn | undefined
  ): vscode.ViewColumn {
    switch (column) {
      case vscode.ViewColumn.One:
        return vscode.ViewColumn.Two;
      case vscode.ViewColumn.Two:
        return vscode.ViewColumn.Three;
      default:
        return vscode.ViewColumn.Beside;
    }
  }

  private getDocumentLabel(document: vscode.TextDocument): string {
    const segments = document.uri.path.split('/');
    return segments[segments.length - 1] || document.fileName || 'Untitled';
  }

  private registerPreview(entry: PreviewEntry): void {
    this.panelsByDocument.set(entry.documentUri, entry);
    this.panelsBySourceColumn.set(String(entry.sourceColumn), entry);
  }

  private unregisterPreview(entry: PreviewEntry): void {
    this.panelsByDocument.delete(entry.documentUri);
    this.panelsBySourceColumn.delete(String(entry.sourceColumn));
    this.renderVersions.delete(entry.documentUri);
  }

  private bindPreviewToDocument(
    entry: PreviewEntry,
    document: vscode.TextDocument,
    sourceColumn: vscode.ViewColumn,
    previewColumn: vscode.ViewColumn
  ): void {
    const nextKey = document.uri.toString();
    const previousDocumentUri = entry.documentUri;
    const previousSourceKey = String(entry.sourceColumn);

    if (previousDocumentUri !== nextKey) {
      this.panelsByDocument.delete(previousDocumentUri);
      this.renderVersions.delete(previousDocumentUri);
    }

    if (previousSourceKey !== String(sourceColumn)) {
      this.panelsBySourceColumn.delete(previousSourceKey);
    }

    entry.documentUri = nextKey;
    entry.sourceColumn = sourceColumn;
    entry.previewColumn = previewColumn;
    this.registerPreview(entry);
  }

  private getPreviewIconPath(): { light: vscode.Uri; dark: vscode.Uri } {
    return {
      light: vscode.Uri.joinPath(this.context.extensionUri, 'media', 'preview-light.svg'),
      dark: vscode.Uri.joinPath(this.context.extensionUri, 'media', 'preview-dark.svg'),
    };
  }

  private resolveSourceColumn(
    column: vscode.ViewColumn | undefined
  ): vscode.ViewColumn {
    return column ?? vscode.ViewColumn.One;
  }

  private isLatestRender(
    key: string,
    version: number,
    panel: vscode.WebviewPanel
  ): boolean {
    const entry = this.panelsByDocument.get(key);
    return entry?.panel === panel && this.renderVersions.get(key) === version;
  }

  private setDiagnostics(
    document: vscode.TextDocument,
    errors: ParseError[]
  ): void {
    const diagnostics = errors.map((error) => this.toDiagnostic(document, error));
    this.diagnostics.set(document.uri, diagnostics);
  }
}

function injectBeforeClosingTag(
  html: string,
  tagName: string,
  content: string
): string {
  const closingTag = new RegExp(`</${tagName}>`, 'i');

  if (closingTag.test(html)) {
    return html.replace(closingTag, `${content}\n</${tagName}>`);
  }

  return `${content}\n${html}`;
}

function ensureMainWrapper(html: string): string {
  if (/<main[\s>]/i.test(html)) {
    return html;
  }

  const bodyTag = /<body([^>]*)>/i;

  if (!bodyTag.test(html)) {
    return html;
  }

  const withMainOpen = html.replace(bodyTag, '<body$1>\n<main>');
  return withMainOpen.replace(/<\/body>/i, '</main>\n</body>');
}

function injectAfterOpeningBody(html: string, content: string): string {
  if (!content) {
    return html;
  }

  const bodyTag = /<body([^>]*)>/i;

  if (bodyTag.test(html)) {
    return html.replace(bodyTag, `<body$1>\n${content}`);
  }

  return `${content}\n${html}`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}
