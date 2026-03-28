import * as os from 'node:os';
import * as path from 'node:path';
import { readFileSync } from 'node:fs';
import * as vscode from 'vscode';
import type { Document as EtchDocument, FrontmatterValue } from './etch-kit/index.js';

const PLUGIN_RELOAD_DEBOUNCE_MS = 150;

type PluginDeclaration =
  | string
  | {
      name: string;
      config?: Record<string, unknown>;
    };

type EtchConfig = {
  plugins: PluginDeclaration[];
  theme: string;
};

type Pipeline = unknown;

type PipelineModule = {
  loadConfig(projectRoot: string): EtchConfig;
  mergeWithFrontmatter(
    config: EtchConfig,
    frontmatter: Record<string, unknown>
  ): EtchConfig;
  discoverPlugins(
    pluginNames: string[],
    projectRoot: string,
    globalRoot: string
  ): Promise<unknown[]>;
  createPipeline(
    plugins: unknown[],
    options?: {
      projectRoot?: string;
      pluginConfig?: Record<string, Record<string, unknown>>;
      log?: (message: string) => void;
    }
  ): Promise<Pipeline>;
  runPipeline(
    html: string,
    document: unknown,
    pipeline: Pipeline,
    config: EtchConfig
  ): Promise<string>;
};

type ThemeDefinition = {
  variables: Record<string, string>;
  darkMode?: {
    variables: Record<string, string>;
  };
};

const BUILTIN_THEMES: Record<string, ThemeDefinition> = {
  default: {
    variables: {
      '--etch-bg': '#ffffff',
      '--etch-text': '#1a1a1a',
      '--etch-heading-font': 'Georgia, serif',
      '--etch-body-font': 'Georgia, serif',
      '--etch-accent': '#2563eb',
      '--etch-code-bg': '#f5f5f5',
    },
    darkMode: {
      variables: {
        '--etch-bg': '#1a1a1a',
        '--etch-text': '#e0e0e0',
        '--etch-code-bg': '#2a2a2a',
      },
    },
  },
  minimal: {
    variables: {
      '--etch-bg': '#ffffff',
      '--etch-text': '#333333',
      '--etch-heading-font':
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      '--etch-body-font':
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      '--etch-accent': '#0066cc',
      '--etch-code-bg': '#f0f0f0',
    },
    darkMode: {
      variables: {
        '--etch-bg': '#1e1e1e',
        '--etch-text': '#d4d4d4',
        '--etch-code-bg': '#2d2d2d',
      },
    },
  },
  academic: {
    variables: {
      '--etch-bg': '#fffff8',
      '--etch-text': '#111111',
      '--etch-heading-font':
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      '--etch-body-font':
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      '--etch-accent': '#8b0000',
      '--etch-code-bg': '#f5f2eb',
    },
    darkMode: {
      variables: {
        '--etch-bg': '#1a1a18',
        '--etch-text': '#d4d0c8',
        '--etch-code-bg': '#2a2820',
      },
    },
  },
};

export class PluginManager implements vscode.Disposable {
  private pipeline: Pipeline | null = null;
  private config: EtchConfig = { plugins: [], theme: 'default' };
  private workspaceRoot: string | null = null;
  private readonly globalRoot = path.join(os.homedir(), '.etch');
  private reloadTimer: ReturnType<typeof setTimeout> | undefined;
  private modulePromise: Promise<PipelineModule> | undefined;

  public async initialize(workspaceRoot: string): Promise<void> {
    this.workspaceRoot = workspaceRoot;
    await this.safeReloadPipeline();
  }

  public async processHtml(
    html: string,
    document: EtchDocument
  ): Promise<string> {
    if (!this.workspaceRoot) {
      return html;
    }

    const normalizedDocument = normalizeValue(document) as Parameters<
      PipelineModule['runPipeline']
    >[1];
    const frontmatterFields = document.frontmatter
      ? (normalizeValue(document.frontmatter.fields) as Record<string, unknown>)
      : undefined;

    if (!this.pipeline) {
      const fallbackConfig = mergeThemeFallbackConfig(
        loadFallbackConfig(this.workspaceRoot),
        frontmatterFields
      );
      return injectThemeOnly(html, fallbackConfig.theme);
    }

    const pipelineModule = await this.getPipelineModule();
    const effectiveConfig = frontmatterFields
      ? pipelineModule.mergeWithFrontmatter(this.config, frontmatterFields)
      : this.config;

    return pipelineModule.runPipeline(
      html,
      normalizedDocument,
      this.pipeline,
      effectiveConfig
    );
  }

  public watchChanges(onReload: () => void): vscode.Disposable {
    if (!this.workspaceRoot) {
      return new vscode.Disposable(() => undefined);
    }

    const patterns = [
      new vscode.RelativePattern(this.workspaceRoot, 'etch.config.json'),
      new vscode.RelativePattern(this.workspaceRoot, '.etch/plugins/**'),
      new vscode.RelativePattern(vscode.Uri.file(this.globalRoot), 'plugins/**'),
    ];

    const watchers = patterns.map((pattern) => {
      const watcher = vscode.workspace.createFileSystemWatcher(pattern);
      const schedule = (): void => {
        this.scheduleReload(onReload);
      };

      watcher.onDidChange(schedule);
      watcher.onDidCreate(schedule);
      watcher.onDidDelete(schedule);
      return watcher;
    });

    return vscode.Disposable.from(...watchers);
  }

  public dispose(): void {
    if (this.reloadTimer) {
      clearTimeout(this.reloadTimer);
      this.reloadTimer = undefined;
    }
  }

  private scheduleReload(onReload: () => void): void {
    if (this.reloadTimer) {
      clearTimeout(this.reloadTimer);
    }

    this.reloadTimer = setTimeout(() => {
      this.reloadTimer = undefined;
      void this.reloadAndNotify(onReload);
    }, PLUGIN_RELOAD_DEBOUNCE_MS);
  }

  private async reloadAndNotify(onReload: () => void): Promise<void> {
    const reloaded = await this.safeReloadPipeline();
    if (reloaded) {
      onReload();
    }
  }

  private async safeReloadPipeline(): Promise<boolean> {
    try {
      await this.reloadPipeline();
      return true;
    } catch (error) {
      console.error('[etch plugins] Failed to reload pipeline', error);
      return false;
    }
  }

  private async reloadPipeline(): Promise<void> {
    if (!this.workspaceRoot) {
      this.pipeline = null;
      this.config = { plugins: [], theme: 'default' };
      return;
    }

    const pipelineModule = await this.getPipelineModule();
    const nextConfig = pipelineModule.loadConfig(this.workspaceRoot);
    const plugins = await pipelineModule.discoverPlugins(
      nextConfig.plugins.map(getPluginName),
      this.workspaceRoot,
      this.globalRoot
    );
    const nextPipeline = await pipelineModule.createPipeline(plugins, {
      projectRoot: this.workspaceRoot,
      pluginConfig: buildPluginConfigMap(nextConfig),
      log: (message) => console.log(`[etch plugins] ${message}`),
    });

    this.config = nextConfig;
    this.pipeline = nextPipeline;
  }

  private async getPipelineModule(): Promise<PipelineModule> {
    if (!this.modulePromise) {
      const moduleUrl = new URL('./vendor/etch-plugin-pipeline/index.js', import.meta.url);
      this.modulePromise = import(moduleUrl.href) as Promise<PipelineModule>;
    }

    return this.modulePromise;
  }
}

function getPluginName(plugin: EtchConfig['plugins'][number]): string {
  return typeof plugin === 'string' ? plugin : plugin.name;
}

function buildPluginConfigMap(
  config: EtchConfig
): Record<string, Record<string, unknown>> {
  const entries = config.plugins
    .filter(
      (
        plugin
      ): plugin is Extract<EtchConfig['plugins'][number], { name: string }> =>
        typeof plugin !== 'string'
    )
    .map((plugin) => [plugin.name, plugin.config ?? {}] as const);

  return Object.fromEntries(entries);
}

function normalizeValue(value: unknown): unknown {
  if (value instanceof Map) {
    return Object.fromEntries(
      Array.from(value.entries(), ([key, entryValue]) => [
        key,
        normalizeValue(entryValue as FrontmatterValue),
      ])
    );
  }

  if (Array.isArray(value)) {
    return value.map((entry) => normalizeValue(entry));
  }

  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, entryValue]) => [
        key,
        normalizeValue(entryValue),
      ])
    );
  }

  return value;
}

function loadFallbackConfig(projectRoot: string): EtchConfig {
  try {
    const raw = readFileSync(path.join(projectRoot, 'etch.config.json'), 'utf8');
    const parsed = JSON.parse(raw) as Partial<EtchConfig>;
    return {
      plugins: Array.isArray(parsed.plugins) ? parsed.plugins : [],
      theme: typeof parsed.theme === 'string' ? parsed.theme : 'default',
    };
  } catch {
    return { plugins: [], theme: 'default' };
  }
}

function mergeThemeFallbackConfig(
  config: EtchConfig,
  frontmatter: Record<string, unknown> | undefined
): EtchConfig {
  return {
    ...config,
    theme:
      frontmatter && typeof frontmatter.theme === 'string'
        ? frontmatter.theme
        : config.theme,
  };
}

function injectThemeOnly(html: string, themeName: string): string {
  const theme = BUILTIN_THEMES[themeName] ?? BUILTIN_THEMES.default;
  const css = assembleThemeCss(theme);
  const tag = `<style data-etch-pipeline="theme">${css}</style>`;

  if (html.includes('</head>')) {
    return html.replace('</head>', `${tag}</head>`);
  }

  return `${tag}${html}`;
}

function assembleThemeCss(theme: ThemeDefinition): string {
  let css = ':root {\n';
  for (const [key, value] of Object.entries(theme.variables)) {
    css += `  ${key}: ${value};\n`;
  }
  css += '}\n';

  if (theme.darkMode) {
    css += '@media (prefers-color-scheme: dark) {\n  :root {\n';
    for (const [key, value] of Object.entries(theme.darkMode.variables)) {
      css += `    ${key}: ${value};\n`;
    }
    css += '  }\n}\n';
  }

  return css;
}
