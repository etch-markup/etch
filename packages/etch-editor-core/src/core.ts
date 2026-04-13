import {
  DEFAULT_STANDALONE_STYLES,
  initialize,
  parseWithErrors,
  renderStandalone,
  type InitializeEtchWasmOptions,
  type ParseError,
} from '@etch-markup/etch-kit';
import {
  BUILTIN_THEMES,
  assembleThemeCSS,
} from '@etch-markup/etch-plugin-pipeline/themes';

const DEFAULT_THEME = 'default';
const DEFAULT_RENDER_DELAY_MS = 200;

export interface EditorState {
  source: string;
  previewHtml: string;
  errors: ParseError[];
  theme: string;
  isInitializing: boolean;
  runtimeError: string | undefined;
}

export interface CreateEtchEditorCoreOptions {
  initialSource?: string | undefined;
  theme?: string | undefined;
  wasmUrl?: string | URL | undefined;
  renderDelayMs?: number | undefined;
}

export interface EtchEditorCore {
  getState(): EditorState;
  setSource(text: string): void;
  setTheme(theme: string): void;
  subscribe(listener: (state: EditorState) => void): () => void;
  dispose(): void;
}

class DefaultEtchEditorCore implements EtchEditorCore {
  private state: EditorState;
  private readonly listeners = new Set<(state: EditorState) => void>();
  private readonly renderDelayMs: number;
  private readonly initializeOptions: InitializeEtchWasmOptions | undefined;
  private renderTimer: ReturnType<typeof setTimeout> | undefined;
  private disposed = false;

  public constructor(options: CreateEtchEditorCoreOptions = {}) {
    this.renderDelayMs = options.renderDelayMs ?? DEFAULT_RENDER_DELAY_MS;
    this.initializeOptions = options.wasmUrl
      ? { wasmUrl: options.wasmUrl }
      : undefined;
    this.state = {
      source: options.initialSource ?? '',
      previewHtml: '',
      errors: [],
      theme: options.theme ?? DEFAULT_THEME,
      isInitializing: true,
      runtimeError: undefined,
    };

    void this.initializeRuntime();
  }

  public getState(): EditorState {
    return this.state;
  }

  public setSource(text: string): void {
    if (this.disposed || text === this.state.source) {
      return;
    }

    this.updateState({ source: text });
    this.scheduleRender();
  }

  public setTheme(theme: string): void {
    if (this.disposed || theme === this.state.theme) {
      return;
    }

    this.updateState({ theme });

    if (!this.state.isInitializing && !this.state.runtimeError) {
      this.renderNow();
    }
  }

  public subscribe(listener: (state: EditorState) => void): () => void {
    if (this.disposed) {
      return () => undefined;
    }

    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  public dispose(): void {
    this.disposed = true;
    if (this.renderTimer) {
      clearTimeout(this.renderTimer);
      this.renderTimer = undefined;
    }
    this.listeners.clear();
  }

  private async initializeRuntime(): Promise<void> {
    try {
      await initialize(this.initializeOptions);

      if (this.disposed) {
        return;
      }

      this.updateState({
        isInitializing: false,
        runtimeError: undefined,
      });
      this.renderNow();
    } catch (error) {
      if (this.disposed) {
        return;
      }

      this.updateState({
        isInitializing: false,
        runtimeError: error instanceof Error ? error.message : String(error),
      });
    }
  }

  private scheduleRender(): void {
    if (this.state.isInitializing || this.state.runtimeError) {
      return;
    }

    if (this.renderTimer) {
      clearTimeout(this.renderTimer);
    }

    this.renderTimer = setTimeout(() => {
      this.renderTimer = undefined;
      this.renderNow();
    }, this.renderDelayMs);
  }

  private renderNow(): void {
    if (this.disposed || this.state.isInitializing || this.state.runtimeError) {
      return;
    }

    if (this.renderTimer) {
      clearTimeout(this.renderTimer);
      this.renderTimer = undefined;
    }

    const { source, theme } = this.state;
    const errors = parseWithErrors(source).errors;
    const previewHtml = renderStandalone(source, buildPreviewStyles(theme));

    this.updateState({
      errors,
      previewHtml,
    });
  }

  private updateState(patch: Partial<EditorState>): void {
    this.state = {
      ...this.state,
      ...patch,
    };

    for (const listener of this.listeners) {
      listener(this.state);
    }
  }
}

function buildPreviewStyles(themeName: string): string {
  const theme = BUILTIN_THEMES[themeName] ?? BUILTIN_THEMES.default;
  const themeCss = theme ? assembleThemeCSS(theme) : '';
  return `${DEFAULT_STANDALONE_STYLES}\n${themeCss}`;
}

export function createEtchEditorCore(
  options: CreateEtchEditorCoreOptions = {}
): EtchEditorCore {
  return new DefaultEtchEditorCore(options);
}
