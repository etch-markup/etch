import { parse, HTMLElement } from "node-html-parser";
import type {
  AstNode,
  DirectiveHandler,
  DirectiveNode,
  Document,
  EtchTheme,
  PluginContext,
  RenderContext
} from "@etch-markup/etch-plugin-sdk";
import { isReservedBuiltinDirectiveName } from "@etch-markup/etch-plugin-sdk";
import type { EtchConfig } from "./config.js";
import { renderFallback } from "./fallback.js";
import type { ResolvedPlugin } from "./discovery.js";
import { BUILTIN_THEMES, assembleThemeCSS } from "./themes.js";

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
      if (isReservedBuiltinDirectiveName(directiveName)) {
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
    const name = element.attributes["data-etch-directive"] ?? "";
    const etchKind = element.attributes["data-etch-kind"] ?? "block";
    const etchId = element.attributes["data-etch-id"] ?? "0";
    const etchContent = element.attributes["data-etch-content"] ?? "";
    const etchAttrs = element.attributes["data-etch-attrs"];
    const kind = etchKind as
      | "inline"
      | "block"
      | "container";
    const directiveId = Number(etchId);
    const originalInnerHtml = element.innerHTML;
    const indexedNode = directiveIndex.get(directiveId);
    const directiveNode: DirectiveNode = indexedNode ?? {
      id: directiveId,
      kind,
      name,
      content: etchContent,
      attributes: parseAttributesJson(etchAttrs),
      children: [],
      span: {
        start: { line: 1, column: 1 },
        end: { line: 1, column: 1 }
      }
    };

    const rawLabel = element.attributes["data-etch-label"];
    if (!indexedNode && typeof rawLabel === "string") {
      directiveNode.rawLabel = rawLabel;
    }

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
    const directiveNode = toDirectiveNode(node);
    if (directiveNode) {
      index.set(directiveNode.id, directiveNode);
    }

    visitChildNodes(node, visit);
  };

  for (const node of body) {
    visit(node);
  }

  return index;
}

type IndexedDirectiveAstNode = AstNode & {
  type: "InlineDirective" | "BlockDirective" | "ContainerDirective";
  directive_id: number;
  name: string;
  label?: AstNode[];
  raw_label?: string;
  raw_content?: string;
  raw_body?: string;
  attrs?: {
    id?: string;
    classes?: string[];
    pairs?: Record<string, string>;
  };
  body?: AstNode[];
  named_close?: boolean;
  span?: DirectiveNode["span"];
};

function toDirectiveNode(node: AstNode): DirectiveNode | undefined {
  if (!isDirectiveNode(node)) {
    return undefined;
  }

  const directiveNode: DirectiveNode = {
    id: node.directive_id,
    kind: getDirectiveKind(node.type),
    name: node.name,
    content: getDirectiveContent(node),
    attributes: flattenAttributes(node.attrs),
    children: getDirectiveChildren(node),
    span: normalizeSpan(node.span)
  };

  if (Array.isArray(node.label)) {
    directiveNode.label = node.label;
  }

  if (typeof node.raw_label === "string") {
    directiveNode.rawLabel = node.raw_label;
  }

  if (node.type === "ContainerDirective" && typeof node.named_close === "boolean") {
    directiveNode.namedClose = node.named_close;
  }

  return directiveNode;
}

function getDirectiveKind(type: IndexedDirectiveAstNode["type"]): DirectiveNode["kind"] {
  if (type === "InlineDirective") {
    return "inline";
  }

  if (type === "BlockDirective") {
    return "block";
  }

  return "container";
}

function getDirectiveContent(node: IndexedDirectiveAstNode): string {
  return node.type === "InlineDirective"
    ? node.raw_content ?? ""
    : node.raw_body ?? "";
}

function getDirectiveChildren(node: IndexedDirectiveAstNode): AstNode[] {
  return node.type === "InlineDirective" || !Array.isArray(node.body)
    ? []
    : node.body;
}

function visitChildNodes(
  node: AstNode,
  visit: (node: AstNode) => void
): void {
  visitNestedAstValue(node, visit, true);
}

function visitNestedAstValue(
  value: unknown,
  visit: (node: AstNode) => void,
  skipCurrentNode = false
): void {
  if (Array.isArray(value)) {
    for (const entry of value) {
      visitNestedAstValue(entry, visit);
    }
    return;
  }

  if (!value || typeof value !== "object") {
    return;
  }

  if (isAstNodeLike(value) && !skipCurrentNode) {
    visit(value);
    return;
  }

  for (const entry of Object.values(value)) {
    visitNestedAstValue(entry, visit);
  }
}

function isAstNodeLike(value: unknown): value is AstNode {
  return (
    !!value &&
    typeof value === "object" &&
    typeof (value as { type?: unknown }).type === "string"
  );
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
