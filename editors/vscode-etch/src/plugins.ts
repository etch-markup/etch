import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import {
  createPipeline,
  discoverPlugins,
  loadConfig,
  mergeWithFrontmatter,
  runPipeline,
  type EtchConfig,
  type Pipeline,
} from '@etch-markup/etch-plugin-pipeline';
import type { Document as EtchDocument, FrontmatterValue } from './etch-kit/index.js';

const PLUGIN_RELOAD_DEBOUNCE_MS = 150;

export class PluginManager implements vscode.Disposable {
  private pipeline: Pipeline | null = null;
  private config: EtchConfig = { plugins: [], theme: 'default' };
  private workspaceRoot: string | null = null;
  private readonly globalRoot = path.join(os.homedir(), '.etch');
  private reloadTimer: ReturnType<typeof setTimeout> | undefined;

  public async initialize(workspaceRoot: string): Promise<void> {
    this.workspaceRoot = workspaceRoot;
    await this.safeReloadPipeline();
  }

  public async processHtml(
    html: string,
    document: EtchDocument
  ): Promise<string> {
    if (!this.pipeline || !this.workspaceRoot) {
      return html;
    }

    const normalizedDocument = normalizeValue(document) as Parameters<
      typeof runPipeline
    >[1];
    const effectiveConfig = document.frontmatter
      ? mergeWithFrontmatter(
          this.config,
          normalizeValue(document.frontmatter.fields) as Record<string, unknown>
        )
      : this.config;

    return runPipeline(html, normalizedDocument, this.pipeline, effectiveConfig);
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

    const nextConfig = loadConfig(this.workspaceRoot);
    const plugins = await discoverPlugins(
      nextConfig.plugins.map(getPluginName),
      this.workspaceRoot,
      this.globalRoot
    );
    const nextPipeline = await createPipeline(plugins, {
      projectRoot: this.workspaceRoot,
      pluginConfig: buildPluginConfigMap(nextConfig),
      log: (message) => console.log(`[etch plugins] ${message}`),
    });

    this.config = nextConfig;
    this.pipeline = nextPipeline;
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
