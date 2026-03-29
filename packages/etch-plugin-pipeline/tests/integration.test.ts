import { beforeAll, describe, expect, it } from "vitest";
import type { EtchPlugin } from "@etch-markup/etch-plugin-sdk";
import { initialize, parse, renderStandalone } from "../../etch-kit/src/index.js";
import { createPipeline, runPipeline } from "../src/pipeline.js";

describe("math and extensibility integration", () => {
  beforeAll(async () => {
    await initialize();
  });

  it("preserves core MathML through the plugin pipeline", async () => {
    const source = "The equation :math[\\frac{1}{2}] is simple.";
    const pipeline = await createPipeline([]);

    const result = await runPipeline(
      renderStandalone(source),
      parse(source),
      pipeline,
      { plugins: [], theme: "default" }
    );

    expect(result).toContain("<math xmlns=\"http://www.w3.org/1998/Math/MathML\">");
    expect(result).toContain("<mfrac><mn>1</mn><mn>2</mn></mfrac>");
    expect(result).not.toContain('data-etch-directive="math"');
  });

  it("renders custom directives with plugin handlers", async () => {
    const source = "::card\nHello *team*\n::";
    const plugin: EtchPlugin = {
      name: "cards",
      version: "1.0.0",
      directives: {
        card: {
          type: "block",
          render(node, ctx) {
            return `<aside class="card">${node.content}|${ctx.renderChildren(node.children)}</aside>`;
          }
        }
      }
    };

    const pipeline = await createPipeline([
      { name: "cards", source: "project", module: plugin }
    ]);

    const result = await runPipeline(
      renderStandalone(source),
      parse(source),
      pipeline,
      { plugins: ["cards"], theme: "default" }
    );

    expect(result).toContain('<aside class="card">Hello *team*|<p>Hello <em>team</em></p></aside>');
    expect(result).not.toContain('data-etch-directive="card"');
  });

  it("renders missing plugin fallbacks for unresolved directives", async () => {
    const source = ":unknown[hello]";
    const pipeline = await createPipeline([]);

    const result = await runPipeline(
      renderStandalone(source),
      parse(source),
      pipeline,
      { plugins: [], theme: "default" }
    );

    expect(result).toContain("etch-missing-plugin");
    expect(result).toContain("Unknown directive");
    expect(result).toContain("<code>unknown</code>");
  });

  it("injects the configured theme into final output", async () => {
    const source = "# Title";
    const pipeline = await createPipeline([]);

    const result = await runPipeline(
      renderStandalone(source),
      parse(source),
      pipeline,
      { plugins: [], theme: "academic" }
    );

    expect(result).toContain('data-etch-pipeline="theme"');
    expect(result).toContain("--etch-bg: #fcfcfa;");
    expect(result).toContain("@media (prefers-color-scheme: dark)");
  });
});
