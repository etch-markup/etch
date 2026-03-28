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

  const plugins: PluginDeclarationObject[] = [];

  for (const entry of value) {
    if (typeof entry === "string") {
      plugins.push({ name: entry });
      continue;
    }

    if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
      continue;
    }

    const objectEntry = entry as Record<string, unknown>;
    if (typeof objectEntry.name === "string") {
      if (
        objectEntry.config &&
        typeof objectEntry.config === "object" &&
        !Array.isArray(objectEntry.config)
      ) {
        plugins.push({
          name: objectEntry.name,
          config: objectEntry.config as Record<string, unknown>
        });
      } else {
        plugins.push({ name: objectEntry.name });
      }
      continue;
    }

    const entries = Object.entries(objectEntry);
    if (entries.length !== 1) {
      continue;
    }

    const firstEntry = entries[0];
    if (!firstEntry) {
      return [];
    }

    const [name, pluginConfig] = firstEntry;
    if (pluginConfig && typeof pluginConfig === "object" && !Array.isArray(pluginConfig)) {
      plugins.push({
        name,
        config: pluginConfig as Record<string, unknown>
      });
    } else {
      plugins.push({ name });
    }
  }

  return plugins;
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
