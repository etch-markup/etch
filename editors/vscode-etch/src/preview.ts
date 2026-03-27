import * as vscode from 'vscode';
import {
  parseWithErrors,
  renderStandalone,
  type ParseError,
} from './etch-kit/index.js';

const ETCH_LANGUAGE_ID = 'etch';
const PREVIEW_DEBOUNCE_MS = 200;

type PreviewEntry = {
  documentUri: string;
  panel: vscode.WebviewPanel;
};

export class EtchPreviewManager implements vscode.Disposable {
  private readonly context: vscode.ExtensionContext;
  private readonly diagnostics: vscode.DiagnosticCollection;
  private readonly panels = new Map<string, PreviewEntry>();
  private readonly refreshTimers = new Map<string, ReturnType<typeof setTimeout>>();

  public constructor(
    context: vscode.ExtensionContext,
    diagnostics: vscode.DiagnosticCollection
  ) {
    this.context = context;
    this.diagnostics = diagnostics;
  }

  public async openPreview(toSide: boolean): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;

    if (!document || document.languageId !== ETCH_LANGUAGE_ID) {
      void vscode.window.showInformationMessage(
        'Open an Etch document to preview it.'
      );
      return;
    }

    const key = document.uri.toString();
    const existing = this.panels.get(key);
    const targetColumn = toSide
      ? this.getSideColumn(editor.viewColumn)
      : editor.viewColumn ?? vscode.ViewColumn.Active;

    if (existing) {
      existing.panel.title = this.getPanelTitle(document);
      existing.panel.reveal(targetColumn, true);
      await this.renderPreview(document, existing.panel);
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

    this.panels.set(key, { documentUri: key, panel });

    panel.onDidDispose(() => {
      this.panels.delete(key);
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

    const entry = this.panels.get(key);

    if (entry) {
      entry.panel.dispose();
    }
  }

  public dispose(): void {
    for (const timer of this.refreshTimers.values()) {
      clearTimeout(timer);
    }

    this.refreshTimers.clear();

    for (const entry of this.panels.values()) {
      entry.panel.dispose();
    }

    this.panels.clear();
  }

  private async updateDocument(document: vscode.TextDocument): Promise<void> {
    this.renderDiagnostics(document);

    const entry = this.panels.get(document.uri.toString());

    if (!entry) {
      return;
    }

    await this.renderPreview(document, entry.panel);
  }

  private renderDiagnostics(document: vscode.TextDocument): ParseError[] {
    const result = parseWithErrors(document.getText());
    const diagnostics = result.errors.map((error) =>
      this.toDiagnostic(document, error)
    );

    this.diagnostics.set(document.uri, diagnostics);
    return result.errors;
  }

  private async renderPreview(
    document: vscode.TextDocument,
    panel: vscode.WebviewPanel
  ): Promise<void> {
    try {
      const errors = this.renderDiagnostics(document);
      const rendered = renderStandalone(document.getText());

      panel.title = this.getPanelTitle(document);
      panel.webview.html = this.decorateHtml(panel.webview, rendered, errors);
    } catch (error) {
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
      `<style>
        :root {
          --etch-bg: var(--vscode-editor-background);
          --etch-text: var(--vscode-editor-foreground);
          --etch-accent: var(--vscode-textLink-foreground);
          --etch-code-bg: var(--vscode-textCodeBlock-background);
        }
      </style>`,
    ].join('\n');

    const banner = this.renderErrorBanner(errors);
    const withHead = injectBeforeClosingTag(html, 'head', headInjection);
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
    const endColumn =
      error.column !== undefined
        ? Math.min(startColumn + 1, line.range.end.character)
        : line.range.end.character;
    const range =
      error.column !== undefined
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
