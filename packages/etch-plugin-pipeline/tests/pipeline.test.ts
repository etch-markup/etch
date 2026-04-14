import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  RESERVED_BUILTIN_DIRECTIVE_NAMES,
  type Document,
  type EtchPlugin
} from "@etch-markup/etch-plugin-sdk";
import { loadConfig, mergeWithFrontmatter } from "../src/config.js";
import { discoverPlugins } from "../src/discovery.js";
import { createPipeline, runPipeline } from "../src/pipeline.js";
import { BUILTIN_THEMES, assembleThemeCSS } from "../src/themes.js";

const tempRoots: string[] = [];

describe("config merging", () => {
  it("merges plugins additively and replaces per-plugin config", () => {
    const merged = mergeWithFrontmatter(
      {
        plugins: [
          "math-extended",
          { name: "anthroverse", config: { apiBase: "https://one" } }
        ],
        theme: "default"
      },
      {
        plugins: [
          "math-extended",
          { anthroverse: { apiBase: "https://two", showNsfw: true } }
        ],
        theme: "academic"
      }
    );

    expect(merged.theme).toBe("academic");
    expect(merged.plugins).toEqual([
      "math-extended",
      {
        name: "anthroverse",
        config: { apiBase: "https://two", showNsfw: true }
      }
    ]);
  });

  it("loads default config when project config is absent", () => {
    const root = makeTempRoot();
    expect(loadConfig(root)).toEqual({ plugins: [], theme: "default" });
  });
});

describe("pipeline runtime", () => {
  it("replaces placeholders and injects styles and theme css", async () => {
    const pipeline = await createCardsPipeline();
    const document = createCardDocument();

    const html = `<!DOCTYPE html><html><head></head><body><div data-etch-directive="card" data-etch-kind="block" data-etch-id="1" data-etch-content="hello"><p>fallback</p></div></body></html>`;
    const result = await runPipeline(html, document, pipeline, {
      plugins: [],
      theme: "academic"
    });

    expect(result).toContain('<div class="card">hello|<p>fallback</p></div>');
    expect(result).toContain('data-etch-pipeline="plugins"');
    expect(result).toContain('data-etch-pipeline="theme"');
    expect(result).toContain("--etch-accent");
  });

  it("preserves core MathML output while replacing plugin placeholders", async () => {
    const pipeline = await createCardsPipeline();
    const document = createCardDocument();

    const html = `<!DOCTYPE html><html><head></head><body><math xmlns="http://www.w3.org/1998/Math/MathML"><mfrac><mn>1</mn><mn>2</mn></mfrac></math><div data-etch-directive="card" data-etch-kind="block" data-etch-id="1" data-etch-content="hello"><p>hello</p></div></body></html>`;
    const result = await runPipeline(html, document, pipeline, {
      plugins: [],
      theme: "default"
    });

    expect(result).toContain(
      '<math xmlns="http://www.w3.org/1998/Math/MathML"><mfrac><mn>1</mn><mn>2</mn></mfrac></math>'
    );
    expect(result).toContain('<div class="card">hello|<p>hello</p></div>');
    expect(result).toContain('data-etch-pipeline="plugins"');
  });

  it("uses fallback output when no handler exists", async () => {
    const pipeline = await createPipeline([]);
    const document: Document = { body: [] };
    const html = `<html><head></head><body><span data-etch-directive="unknown" data-etch-kind="inline" data-etch-id="1" data-etch-content=""></span></body></html>`;
    const result = await runPipeline(html, document, pipeline, {
      plugins: [],
      theme: "default"
    });

    expect(result).toContain("etch-missing-plugin");
    expect(result).toContain("Unknown directive");
  });

  it("rejects reserved directive handlers", async () => {
    const plugin: EtchPlugin = {
      name: "bad-plugin",
      version: "1.0.0",
      directives: {
        math: {
          type: "inline",
          render: () => "<span>bad</span>"
        }
      }
    };

    const pipeline = await createPipeline([
      { name: "bad-plugin", source: "project", module: plugin }
    ]);

    expect(pipeline.handlers.has("math")).toBe(false);
    expect(pipeline.warnings).toContain("Ignored reserved directive handler: math");
  });

  it("keeps non-reserved directive names available to plugins", async () => {
    const plugin: EtchPlugin = {
      name: "spoilers",
      version: "1.0.0",
      directives: {
        spoiler: {
          type: "block",
          render: () => '<div class="spoiler-card">revealed</div>'
        }
      }
    };

    const pipeline = await createPipeline([
      { name: "spoilers", source: "project", module: plugin }
    ]);

    expect(pipeline.handlers.has("spoiler")).toBe(true);
    expect(RESERVED_BUILTIN_DIRECTIVE_NAMES).not.toContain("spoiler");
  });

  it("passes raw labels, named closes, and nested directive nodes to plugin handlers", async () => {
    let receivedNode:
      | {
          rawLabel?: string;
          namedClose?: boolean;
          label?: unknown[];
          children: unknown[];
        }
      | undefined;

    const plugin: EtchPlugin = {
      name: "chapters",
      version: "1.0.0",
      directives: {
        panel: {
          type: "container",
          render(node, ctx) {
            receivedNode = node;
            return `<section class="chapter">${node.rawLabel}|${node.namedClose}|${ctx.renderChildren(
              node.children
            )}</section>`;
          }
        }
      }
    };

    const pipeline = await createPipeline([
      { name: "chapters", source: "project", module: plugin }
    ]);

    const document: Document = {
      body: [
        {
          type: "ContainerDirective",
          directive_id: 7,
          name: "panel",
          label: [{ type: "Text", value: "Part One" }],
          raw_label: "Part One",
          raw_body: "Paragraph",
          named_close: true,
          body: [
            {
              type: "Table",
              headers: [
                {
                  content: [
                    {
                      type: "InlineDirective",
                      directive_id: 9,
                      name: "callout",
                      raw_content: "nested",
                      attrs: { pairs: { tone: "warm" } },
                      span: {
                        start: { line: 2, column: 1 },
                        end: { line: 2, column: 16 }
                      }
                    }
                  ]
                }
              ],
              rows: [],
              alignments: []
            }
          ],
          span: {
            start: { line: 1, column: 1 },
            end: { line: 3, column: 12 }
          }
        }
      ]
    };

    const html =
      '<!DOCTYPE html><html><head></head><body><section data-etch-directive="panel" data-etch-kind="container" data-etch-id="7" data-etch-content="Paragraph" data-etch-label="Part One"><p>fallback</p></section></body></html>';

    const result = await runPipeline(html, document, pipeline, {
      plugins: [],
      theme: "default"
    });

    expect(result).toContain('<section class="chapter">Part One|true|<p>fallback</p></section>');
    expect(receivedNode).toMatchObject({
      id: 7,
      kind: "container",
      name: "panel",
      rawLabel: "Part One",
      namedClose: true,
      label: [{ type: "Text", value: "Part One" }],
      children: [
        {
          type: "Table",
          headers: [
            {
              content: [
                {
                  type: "InlineDirective",
                  directive_id: 9
                }
              ]
            }
          ]
        }
      ]
    });
  });
});

describe("discovery", () => {
  it("loads plugins from project before global and strips reserved handlers", async () => {
    const projectRoot = makeTempRoot();
    const globalRoot = makeTempRoot();
    writePlugin(projectRoot, ".etch/plugins/etch-plugin-cards", {
      name: "cards",
      version: "1.0.0",
      directives: {
        math: { type: "inline", render: "() => '<span>bad</span>'" },
        card: { type: "block", render: "() => '<div>project</div>'" }
      }
    });
    writePlugin(globalRoot, "plugins/etch-plugin-cards", {
      name: "cards",
      version: "2.0.0",
      directives: {
        card: { type: "block", render: "() => '<div>global</div>'" }
      }
    });

    const plugins = await discoverPlugins(["cards"], projectRoot, globalRoot);

    expect(plugins).toHaveLength(1);
    expect(plugins[0].source).toBe("project");
    expect(Object.keys(plugins[0].module.directives)).toEqual(["card"]);
  });
});

describe("themes", () => {
  it("assembles built-in themes with dark mode", () => {
    const css = assembleThemeCSS(BUILTIN_THEMES.academic);
    expect(css).toContain("--etch-bg: #fcfcfa;");
    expect(css).toContain("@media (prefers-color-scheme: dark)");
  });

  it("keeps the paper theme locked to a light paper surface", () => {
    const css = assembleThemeCSS(BUILTIN_THEMES.paper);
    expect(css).toContain("--etch-bg: #ffffff;");
    expect(css).toContain("color-scheme: light;");
  });
});

afterEach(() => {
  while (tempRoots.length > 0) {
    const path = tempRoots.pop();
    if (path) {
      // Let the OS clean up tmp dirs; tests only need isolated paths during the run.
    }
  }
});

function makeTempRoot(): string {
  const root = mkdtempSync(join(tmpdir(), "etch-pipeline-"));
  tempRoots.push(root);
  return root;
}

async function createCardsPipeline() {
  return createPipeline([
    { name: "cards", source: "project", module: createCardPlugin() }
  ]);
}

function createCardPlugin(): EtchPlugin {
  return {
    name: "cards",
    version: "1.0.0",
    directives: {
      card: {
        type: "block",
        styles: ".card{color:var(--etch-accent);}",
        render(node, ctx) {
          return `<div class="card">${node.content}|${ctx.renderChildren(node.children)}</div>`;
        }
      }
    }
  };
}

function createCardDocument(): Document {
  return {
    body: [
      {
        type: "BlockDirective",
        directive_id: 1,
        name: "card",
        raw_body: "hello",
        attrs: { pairs: { tone: "warm" } },
        body: [{ type: "Paragraph" }],
        span: {
          start: { line: 1, column: 1 },
          end: { line: 3, column: 3 }
        }
      }
    ]
  };
}

function writePlugin(
  root: string,
  relativeDir: string,
  plugin: {
    name: string;
    version: string;
    directives: Record<string, { type: string; render: string }>;
  }
): void {
  const pluginRoot = join(root, relativeDir);
  mkdirSync(pluginRoot, { recursive: true });
  writeFileSync(
    join(pluginRoot, "package.json"),
    JSON.stringify(
      {
        name: `etch-plugin-${plugin.name}`,
        version: plugin.version,
        type: "module",
        main: "index.js"
      },
      null,
      2
    )
  );

  const directiveEntries = Object.entries(plugin.directives)
    .map(
      ([name, value]) =>
        `"${name}": { type: "${value.type}", render: ${value.render} }`
    )
    .join(",\n");

  writeFileSync(
    join(pluginRoot, "index.js"),
    `export default { name: "${plugin.name}", version: "${plugin.version}", directives: { ${directiveEntries} } };`
  );
}
