import { access, readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import type { EtchPlugin } from "@etch-markup/etch-plugin-sdk";

const RESERVED_DIRECTIVES = new Set([
  "math",
  "note",
  "aside",
  "figure",
  "details",
  "section",
  "chapter",
  "columns",
  "column",
  "pagebreak",
  "toc",
  "abbr",
  "cite",
  "kbd"
]);

export interface ResolvedPlugin {
  name: string;
  module: EtchPlugin;
  source: "project" | "global";
}

export async function discoverPlugins(
  pluginNames: string[],
  projectRoot: string,
  globalRoot: string
): Promise<ResolvedPlugin[]> {
  const resolved: ResolvedPlugin[] = [];

  for (const name of pluginNames) {
    const projectCandidate = join(
      projectRoot,
      ".etch/plugins",
      normalizedPackageName(name)
    );
    const globalCandidate = join(
      globalRoot,
      "plugins",
      normalizedPackageName(name)
    );

    if (await pathExists(projectCandidate)) {
      const plugin = await loadResolvedPlugin(projectCandidate, "project");
      if (plugin) {
        resolved.push(plugin);
      }
      continue;
    }

    if (await pathExists(globalCandidate)) {
      const plugin = await loadResolvedPlugin(globalCandidate, "global");
      if (plugin) {
        resolved.push(plugin);
      }
    }
  }

  return resolved;
}

export function normalizedPackageName(name: string): string {
  return name.startsWith("etch-plugin-") ? name : `etch-plugin-${name}`;
}

async function loadResolvedPlugin(
  root: string,
  source: "project" | "global"
): Promise<ResolvedPlugin | null> {
  const manifest = JSON.parse(
    await readFile(join(root, "package.json"), "utf8")
  ) as {
    name?: string;
    main?: string;
    exports?: string | { import?: string; default?: string };
  };

  const entry =
    (typeof manifest.exports === "string"
      ? manifest.exports
      : manifest.exports?.import ?? manifest.exports?.default) ??
    manifest.main ??
    "index.js";

  const imported = (await import(pathToFileURL(join(root, entry)).href)) as {
    default?: EtchPlugin;
  };
  const plugin = imported.default;
  if (!plugin) {
    return null;
  }

  const filteredDirectives = Object.fromEntries(
    Object.entries(plugin.directives).filter(([directiveName]) => {
      if (RESERVED_DIRECTIVES.has(directiveName)) {
        return false;
      }
      return true;
    })
  );

  return {
    name: plugin.name,
    source,
    module: {
      ...plugin,
      directives: filteredDirectives
    }
  };
}

async function pathExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}
