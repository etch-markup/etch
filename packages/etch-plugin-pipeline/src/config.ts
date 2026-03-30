import { readFileSync } from "node:fs";
import { join } from "node:path";

export interface PluginDeclarationObject {
  name: string;
  config?: Record<string, unknown>;
}

export type PluginDeclaration = string | PluginDeclarationObject;

export interface EtchConfig {
  plugins: PluginDeclaration[];
  theme: string;
}

const DEFAULT_CONFIG: EtchConfig = {
  plugins: [],
  theme: "default"
};

export function loadConfig(projectRoot: string): EtchConfig {
  const path = join(projectRoot, "etch.config.json");

  try {
    const raw = readFileSync(path, "utf8");
    const parsed = JSON.parse(raw) as Partial<EtchConfig>;
    return {
      plugins: Array.isArray(parsed.plugins) ? parsed.plugins : [],
      theme: typeof parsed.theme === "string" ? parsed.theme : DEFAULT_CONFIG.theme
    };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

export function mergeWithFrontmatter(
  config: EtchConfig,
  frontmatter: Record<string, unknown>
): EtchConfig {
  const merged = new Map<string, PluginDeclarationObject>();

  for (const plugin of config.plugins) {
    const normalized = normalizePluginDeclaration(plugin);
    merged.set(normalized.name, normalized);
  }

  const frontmatterPlugins = parseFrontmatterPlugins(frontmatter.plugins);
  for (const plugin of frontmatterPlugins) {
    merged.set(plugin.name, plugin);
  }

  return {
    plugins: Array.from(merged.values()).map(compactPluginDeclaration),
    theme:
      typeof frontmatter.theme === "string" ? frontmatter.theme : config.theme
  };
}

function parseFrontmatterPlugins(value: unknown): PluginDeclarationObject[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.flatMap(parseFrontmatterPluginEntry);
}

function normalizePluginDeclaration(plugin: PluginDeclaration): PluginDeclarationObject {
  if (typeof plugin === "string") {
    return { name: plugin };
  }

  return plugin.config
    ? {
        name: plugin.name,
        config: plugin.config
      }
    : {
        name: plugin.name
      };
}

function compactPluginDeclaration(plugin: PluginDeclarationObject): PluginDeclaration {
  return plugin.config ? plugin : plugin.name;
}

function parseFrontmatterPluginEntry(entry: unknown): PluginDeclarationObject[] {
  if (typeof entry === "string") {
    return [{ name: entry }];
  }

  const objectEntry = asRecord(entry);
  if (!objectEntry) {
    return [];
  }

  const namedPlugin = parseNamedPluginDeclaration(objectEntry);
  if (namedPlugin) {
    return [namedPlugin];
  }

  return parseMappedPluginDeclaration(objectEntry);
}

function parseNamedPluginDeclaration(
  entry: Record<string, unknown>
): PluginDeclarationObject | undefined {
  if (typeof entry.name !== "string") {
    return undefined;
  }

  const config = asRecord(entry.config);
  return config
    ? {
        name: entry.name,
        config
      }
    : {
        name: entry.name
      };
}

function parseMappedPluginDeclaration(
  entry: Record<string, unknown>
): PluginDeclarationObject[] {
  const entries = Object.entries(entry);
  if (entries.length !== 1) {
    return [];
  }

  const [name, pluginConfig] = entries[0] ?? [];
  if (typeof name !== "string") {
    return [];
  }

  const config = asRecord(pluginConfig);
  return [
    config
      ? {
          name,
          config
        }
      : {
          name
        }
  ];
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
