export type AstNode = {
  type: string;
  [key: string]: unknown;
};

export interface Document {
  frontmatter?: {
    raw: string;
    fields: Record<string, unknown>;
  };
  body: AstNode[];
}

export interface DirectiveSpan {
  start: {
    line: number;
    column: number;
  };
  end: {
    line: number;
    column: number;
  };
}

export interface DirectiveNode {
  id: number;
  kind: "inline" | "block" | "container";
  name: string;
  content: string;
  attributes: Record<string, string>;
  children: AstNode[];
  span: DirectiveSpan;
}

export interface RenderContext {
  renderChildren(nodes: AstNode[]): string;
  document: Document;
  config: Record<string, unknown>;
  warn(message: string): void;
  error(message: string): void;
}

export interface PluginContext {
  config: Record<string, unknown>;
  projectRoot: string;
  log(message: string): void;
}

export interface Completion {
  label: string;
  detail?: string;
}

export interface EditorSupport {
  completions?(
    partial: string,
    ctx: PluginContext
  ): Completion[] | Promise<Completion[]>;
  hover?(node: DirectiveNode, ctx: PluginContext): string | Promise<string>;
}

export interface DirectiveHandler {
  type: "inline" | "block" | "container" | "all";
  render(node: DirectiveNode, ctx: RenderContext): string | Promise<string>;
  styles?: string;
  editor?: EditorSupport;
}

export interface EtchTheme {
  name: string;
  variables: {
    "--etch-bg": string;
    "--etch-text": string;
    "--etch-heading-font": string;
    "--etch-body-font": string;
    "--etch-accent": string;
    "--etch-code-bg": string;
    [key: `--etch-${string}`]: string;
  };
  css?: string;
  darkMode?: {
    variables: Record<string, string>;
    css?: string;
  };
}

export interface EtchPlugin {
  name: string;
  version: string;
  directives: Record<string, DirectiveHandler>;
  themes?: Record<string, EtchTheme>;
  setup?(ctx: PluginContext): Promise<void>;
  teardown?(): Promise<void>;
}
