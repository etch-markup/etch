import { parse, HTMLElement } from "node-html-parser";
import type {
  AstNode,
  DirectiveHandler,
  DirectiveNode,
  Document,
  EtchPlugin,
  EtchTheme,
  PluginContext,
  RenderContext
} from "@etch-markup/etch-plugin-sdk";
import type { EtchConfig } from "./config.js";
import { renderFallback } from "./fallback.js";
import type { ResolvedPlugin } from "./discovery.js";
import { BUILTIN_THEMES, assembleThemeCSS } from "./themes.js";

const RESERVED_DIRECTIVES = new Set([
  "math",
  "note",
  "aside",
  "figure",
  "details",
  "spoiler",
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

export interface Pipeline {
  handlers: Map<string, { plugin: string; handler: DirectiveHandler }>;
  styles: string[];
  themes: Map<string, EtchTheme>;
  warnings: string[];
}

export interface CreatePipelineOptions {
  projectRoot?: string;
  pluginConfig?: Record<string, Record<string, unknown>>;
  log?: (message: string) => void;
}

export async function createPipeline(
  plugins: ResolvedPlugin[],
  options: CreatePipelineOptions = {}
): Promise<Pipeline> {
  const handlers = new Map<string, { plugin: string; handler: DirectiveHandler }>();
  const styles = new Set<string>();
  const themes = new Map<string, EtchTheme>();
  const warnings: string[] = [];

  for (const { module } of plugins) {
    const pluginContext: PluginContext = {
      config: options.pluginConfig?.[module.name] ?? {},
      projectRoot: options.projectRoot ?? process.cwd(),
      log: (message) => options.log?.(`[${module.name}] ${message}`)
    };

    await module.setup?.(pluginContext);

    for (const [directiveName, handler] of Object.entries(module.directives)) {
      if (RESERVED_DIRECTIVES.has(directiveName)) {
        warnings.push(`Ignored reserved directive handler: ${directiveName}`);
        continue;
      }
      handlers.set(directiveName, { plugin: module.name, handler });
      if (handler.styles) {
        styles.add(handler.styles);
      }
    }

    for (const [themeName, theme] of Object.entries(module.themes ?? {})) {
      themes.set(themeName, theme);
    }
  }

  return {
    handlers,
    styles: Array.from(styles),
    themes,
    warnings
  };
}

export async function runPipeline(
  html: string,
  document: Document,
  pipeline: Pipeline,
  config: EtchConfig
): Promise<string> {
  const root = parse(html);
  const directiveIndex = buildDirectiveIndex(document.body);

  for (const element of root.querySelectorAll("[data-etch-directive]")) {
    const name = element.getAttribute("data-etch-directive") ?? "";
    const kind = (element.getAttribute("data-etch-kind") ?? "block") as
      | "inline"
      | "block"
      | "container";
    const directiveId = Number(element.getAttribute("data-etch-id") ?? "0");
    const originalInnerHtml = element.innerHTML;
    const indexedNode = directiveIndex.get(directiveId);
    const directiveNode: DirectiveNode = indexedNode ?? {
      id: directiveId,
      kind,
      name,
      content: element.getAttribute("data-etch-content") ?? "",
      attributes: parseAttributesJson(element.getAttribute("data-etch-attrs")),
      children: [],
      span: {
        start: { line: 1, column: 1 },
        end: { line: 1, column: 1 }
      }
    };

    const registered = pipeline.handlers.get(name);
    if (!registered) {
      element.replaceWith(renderFallback(name, kind));
      continue;
    }

    const renderContext: RenderContext = {
      renderChildren: () => originalInnerHtml,
      document,
      config: getPluginConfig(config, registered.plugin),
      warn: (message) => pipeline.warnings.push(`[${registered.plugin}] ${message}`),
      error: (message) => pipeline.warnings.push(`[${registered.plugin}] ${message}`)
    };

    const rendered = await registered.handler.render(directiveNode, renderContext);
    element.replaceWith(rendered);
  }

  injectStyles(root, pipeline.styles);
  injectTheme(root, pipeline, config.theme);

  return root.toString();
}

function getPluginConfig(
  config: EtchConfig,
  pluginName: string
): Record<string, unknown> {
  for (const plugin of config.plugins) {
    if (typeof plugin === "string") {
      continue;
    }

    if (plugin.name === pluginName) {
      return plugin.config ?? {};
    }
  }

  return {};
}

function buildDirectiveIndex(body: AstNode[]): Map<number, DirectiveNode> {
  const index = new Map<number, DirectiveNode>();
  const visit = (node: AstNode): void => {
    if (isDirectiveNode(node)) {
      index.set(node.directive_id, {
        id: node.directive_id,
        kind: node.type === "InlineDirective"
          ? "inline"
          : node.type === "BlockDirective"
            ? "block"
            : "container",
        name: node.name,
        content:
          node.type === "InlineDirective"
            ? node.raw_content ?? ""
            : node.raw_body ?? "",
        attributes: flattenAttributes(node.attrs),
        children:
          node.type === "InlineDirective"
            ? []
            : Array.isArray(node.body)
              ? node.body
              : [],
        span: normalizeSpan(node.span)
      });
    }

    const nodeWithBody = node as unknown as { body?: AstNode[] };
    if (Array.isArray(nodeWithBody.body)) {
      for (const child of nodeWithBody.body) {
        visit(child);
      }
    }

    const nodeWithContent = node as unknown as { content?: AstNode[] };
    if (Array.isArray(nodeWithContent.content)) {
      for (const child of nodeWithContent.content) {
        visit(child);
      }
    }

    const nodeWithItems = node as unknown as { items?: AstNode[] };
    if (Array.isArray(nodeWithItems.items)) {
      for (const child of nodeWithItems.items) {
        visit(child);
      }
    }
  };

  for (const node of body) {
    visit(node);
  }

  return index;
}

function isDirectiveNode(node: AstNode): node is AstNode & {
  type: "InlineDirective" | "BlockDirective" | "ContainerDirective";
  directive_id: number;
  name: string;
  raw_content?: string;
  raw_body?: string;
  attrs?: {
    id?: string;
    classes?: string[];
    pairs?: Record<string, string>;
  };
  body?: AstNode[];
  span?: DirectiveNode["span"];
} {
  const type = node.type;
  return (
    (type === "InlineDirective" ||
      type === "BlockDirective" ||
      type === "ContainerDirective") &&
    typeof (node as { directive_id?: unknown }).directive_id === "number" &&
    typeof (node as { name?: unknown }).name === "string"
  );
}

function flattenAttributes(
  attrs:
    | {
        id?: string;
        classes?: string[];
        pairs?: Record<string, string> | Map<string, string>;
      }
    | undefined
): Record<string, string> {
  if (!attrs) {
    return {};
  }

  const flattened: Record<string, string> = {};
  if (attrs.id) {
    flattened.id = attrs.id;
  }
  if (attrs.classes?.length) {
    flattened.class = attrs.classes.join(" ");
  }
  if (attrs.pairs instanceof Map) {
    for (const [key, value] of attrs.pairs.entries()) {
      flattened[key] = value;
    }
  } else if (attrs.pairs) {
    Object.assign(flattened, attrs.pairs);
  }
  return flattened;
}

function normalizeSpan(span: DirectiveNode["span"] | undefined): DirectiveNode["span"] {
  return (
    span ?? {
      start: { line: 1, column: 1 },
      end: { line: 1, column: 1 }
    }
  );
}

function parseAttributesJson(value: string | null | undefined): Record<string, string> {
  if (!value) {
    return {};
  }

  try {
    const parsed = JSON.parse(value) as unknown;
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as Record<string, string>)
      : {};
  } catch {
    return {};
  }
}

function injectStyles(root: HTMLElement, styles: string[]): void {
  if (styles.length === 0) {
    return;
  }

  const target = root.querySelector("head") ?? root;
  target.insertAdjacentHTML(
    "beforeend",
    `<style data-etch-pipeline="plugins">${styles.join("\n")}</style>`
  );
}

function injectTheme(root: HTMLElement, pipeline: Pipeline, activeThemeName: string): void {
  const theme =
    pipeline.themes.get(activeThemeName) ??
    BUILTIN_THEMES[activeThemeName] ??
    BUILTIN_THEMES.default;
  if (!theme) {
    return;
  }
  const css = assembleThemeCSS(theme);
  const target = root.querySelector("head") ?? root;
  target.insertAdjacentHTML(
    "beforeend",
    `<style data-etch-pipeline="theme">${css}</style>`
  );
}
