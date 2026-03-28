# 10. Math Support & Extensibility Model

This specification defines Etch's core math rendering system, its plugin-based extensibility model, and how the two integrate across CLI, IDE, and future WYSIWYG editor surfaces.

---

## 1. Overview

Etch's extensibility is built on two pillars:

1. **Directives as the parser-level extension point.** The parser already handles inline (`:name[content]{attrs}`), block (`::name ... ::`), and container (`:::name ... :::`) directives generically. Unknown directive names are preserved in the AST without parser modification.

2. **A render-time plugin pipeline as the output-level extension point.** Plugins are JS/TS modules that register handlers for directive names. At render time, the pipeline invokes the matching handler for each directive placeholder and substitutes its output into the final HTML.

Math is the first feature to exercise both pillars: basic math is a core, built-in capability (Tier 1), while advanced math constructs are delivered as a first-party plugin (Tier 2). Third-party plugins (e.g., anthroverse) use the same system.

### 1.1 Design Goals

- **Zero-config basics.** Core math renders with no plugins, no JS dependencies, no configuration. Write `:math[\alpha]`, get a rendered alpha.
- **Accessible plugin authoring.** Plugins are JS/TS — no Rust, no WASM compilation. A plugin is a single module exporting a well-typed object from `@etch-markup/etch-plugin-sdk`.
- **Portable plugins.** One plugin works identically across CLI, VS Code, JetBrains, Neovim, and the future WYSIWYG editor. The render pipeline is pure JS, not tied to any editor.
- **Smooth user experience.** End users interact with `etch add <name>` and `etch remove <name>`. No manual npm workflow is required in their project.
- **Graceful degradation.** Documents using unavailable plugins render with clear, actionable warnings — never silent failures.
- **Theme-aware.** Plugins read from a shared CSS variable namespace (`--etch-*`). Switching themes re-skins core and plugin output alike.

### 1.2 Tiers

| Tier | Scope | Delivery | Example |
|------|-------|----------|---------|
| **Tier 1 — Core** | Built into `etch-core` (Rust). Zero config. | Ships with every Etch installation. | Greek letters, fractions, integrals, roots, sub/superscript → MathML |
| **Tier 2 — Plugin** | JS/TS modules loaded at render time. | Installed via `etch add <name>`. | `math-extended` (matrices, aligned equations), `anthroverse`, `diagrams` |
| **Tier 3 — Editor UI** | Same plugins, surfaced through a GUI. | Click-to-enable in the WYSIWYG editor. | Future: plugin browser in the Etch editor application |

---

## 2. Core Math (Tier 1)

### 2.1 Syntax

Math uses the existing directive syntax — no new syntax forms are introduced.

**Inline math:**

```etch
The equation :math[E = mc^2] is well-known.
```

**Display math (block):**

```etch
::math
\int_0^1 x^2 \, dx = \frac{1}{3}
::
```

### 2.2 Supported LaTeX Subset

Core math handles a subset of LaTeX math mode sufficient for typical STEM authoring:

| Category | Commands | Example |
|----------|----------|---------|
| Greek letters | `\alpha` `\beta` `\gamma` `\delta` `\epsilon` `\theta` `\lambda` `\mu` `\pi` `\sigma` `\omega` (and uppercase variants) | `\alpha + \beta` |
| Fractions | `\frac{num}{den}` | `\frac{1}{2}` |
| Roots | `\sqrt{x}` `\sqrt[n]{x}` | `\sqrt[3]{8}` |
| Superscript / Subscript | `^{...}` `_{...}` | `x^{2}` `a_{i,j}` |
| Summation / Product | `\sum` `\prod` with `_{...}^{...}` limits | `\sum_{i=0}^{n}` |
| Integrals | `\int` `\iint` `\iiint` `\oint` with limits | `\int_0^1 f(x)\,dx` |
| Operators | `\pm` `\times` `\div` `\cdot` `\leq` `\geq` `\neq` `\approx` `\equiv` `\in` `\notin` `\subset` `\cup` `\cap` | `a \neq b` |
| Arrows | `\to` `\rightarrow` `\leftarrow` `\Rightarrow` `\Leftrightarrow` | `f: A \to B` |
| Delimiters | `\left(` `\right)` `\left[` `\right]` `\left\{` `\right\}` | `\left(\frac{1}{2}\right)` |
| Spacing | `\,` `\;` `\quad` `\qquad` | `a \quad b` |
| Text in math | `\text{...}` | `\text{if } x > 0` |
| Accents | `\hat{x}` `\bar{x}` `\vec{v}` `\dot{x}` `\tilde{x}` | `\vec{v}` |
| Fonts | `\mathbb{R}` `\mathcal{L}` `\mathbf{v}` | `\mathbb{R}^n` |

This list is intentionally bounded. Constructs beyond this scope (matrices, environments, aligned equations) belong in the `math-extended` first-party plugin.

### 2.3 Output: MathML

Core math renders to MathML, a W3C standard with Baseline browser support since 2024. This means:

- **Zero JS dependency.** Browsers render MathML natively.
- **Accessible by default.** Screen readers can interpret MathML semantics.
- **Consistent output.** CLI `etch render` and IDE preview produce identical markup.

Example — `:math[\frac{1}{2}]` renders to:

```html
<math xmlns="http://www.w3.org/1998/Math/MathML">
  <mfrac>
    <mn>1</mn>
    <mn>2</mn>
  </mfrac>
</math>
```

Display math (`::math ... ::`) wraps in `<math display="block">` for centered, full-width rendering.

### 2.4 Implementation Location

The LaTeX-to-MathML conversion lives in `etch-core` (Rust), invoked during HTML rendering. When the renderer encounters a directive named `math`:

1. Extract the raw LaTeX string from the directive content.
2. Parse the LaTeX subset into an intermediate representation.
3. Emit MathML markup.
4. Embed the MathML inline in the HTML output.

This runs in WASM for browser/IDE contexts and natively for CLI. The `math` directive is **not** passed to the plugin pipeline — it is fully handled by core.

**Reserved directive names.** Core directive names (currently: `math`) are reserved. Plugins MUST NOT register handlers for reserved names. If a plugin attempts to register a handler for a reserved name, the runtime MUST ignore the handler and emit a warning. This prevents plugins from silently shadowing core behavior.

---

## 3. Plugin System (Tier 2)

### 3.1 What Is a Plugin

A plugin is a JS/TS module that default-exports an `EtchPlugin` object. It declares which directives it handles, how to render them, and optionally provides styles, themes, and editor intelligence.

### 3.2 Plugin Interface

```typescript
interface EtchPlugin {
  /** Unique plugin name. Matches the name used in `etch add <name>`. */
  name: string;

  /** Semver version string. */
  version: string;

  /** Map of directive names to their handlers. */
  directives: Record<string, DirectiveHandler>;

  /** Optional themes this plugin provides. */
  themes?: Record<string, EtchTheme>;

  /** Called once when the plugin is loaded. Use for initialization (API clients, caches, etc.). */
  setup?(ctx: PluginContext): Promise<void>;

  /** Called when the plugin is unloaded. Use for cleanup. */
  teardown?(): Promise<void>;
}
```

### 3.3 Directive Handler

```typescript
interface DirectiveHandler {
  /** Which directive forms this handler accepts. */
  type: "inline" | "block" | "container" | "all";

  /** Transform a directive AST node into HTML. May be async (e.g., fetch from API). */
  render(node: DirectiveNode, ctx: RenderContext): string | Promise<string>;

  /** CSS to inject into the output. Collected from all plugins, injected once. */
  styles?: string;

  /** Optional editor integration (autocomplete, hover, diagnostics). */
  editor?: EditorSupport;
}
```

### 3.4 Directive Node

The data structure passed to `render()`:

```typescript
interface DirectiveNode {
  /** Stable directive id assigned during parsing. */
  id: number;

  /** Directive kind. */
  kind: "inline" | "block" | "container";

  /** Directive name, e.g., "anthroverse-char". */
  name: string;

  /** Raw text content from brackets or block body text. */
  content: string;

  /** Parsed attributes from braces, e.g., { species: "dragon" } from {species=dragon}. */
  attributes: Record<string, string>;

  /** Parsed child AST nodes. Inline directives use an empty array. */
  children: AstNode[];

  /** Source span for diagnostics. */
  span: DirectiveSpan;
}
```

### 3.5 Render Context

```typescript
interface RenderContext {
  /** Render child AST nodes to HTML (for container directives that wrap content). */
  renderChildren(nodes: AstNode[]): string;

  /** The full document AST. Useful for cross-references, table of contents, etc. */
  document: Document;

  /** Plugin-specific config from etch.config.json or frontmatter. */
  config: Record<string, any>;

  /** Report a warning (shown in editor diagnostics and CLI output). */
  warn(message: string): void;

  /** Report an error (shown in editor diagnostics and CLI output). */
  error(message: string): void;
}
```

### 3.6 Plugin Context

```typescript
interface PluginContext {
  /** Plugin-specific config from etch.config.json or frontmatter. */
  config: Record<string, any>;

  /** The project root directory (where etch.config.json lives). */
  projectRoot: string;

  /** Log a message (visible in CLI verbose mode and editor output channels). */
  log(message: string): void;
}
```

### 3.7 Editor Support

Optional interface for enriching the IDE experience:

```typescript
interface EditorSupport {
  /** Provide autocomplete suggestions when typing inside a directive's brackets.
   *  Called with the partial text the user has typed so far. */
  completions?(partial: string, ctx: PluginContext): Completion[] | Promise<Completion[]>;

  /** Provide hover information for a directive instance. */
  hover?(node: DirectiveNode, ctx: PluginContext): string | Promise<string>;
}

interface Completion {
  /** The text to insert. */
  label: string;
  /** Optional description shown alongside the suggestion. */
  detail?: string;
}
```

### 3.8 Example Plugin: Anthroverse

```typescript
import type { EtchPlugin } from "@etch-markup/etch-plugin-sdk";

export default {
  name: "anthroverse",
  version: "1.0.0",

  async setup(ctx) {
    // Initialize API client with base URL from config
  },

  directives: {
    "anthroverse-char": {
      type: "inline",
      async render(node, ctx) {
        const name = node.content;
        const species = node.attributes.species ?? ctx.config.defaultSpecies;
        // Fetch character data, return rich HTML
        return `<span class="av-char" data-species="${species}">
          <img src="..." alt="${name}" /> ${name}
        </span>`;
      },
      styles: `
        .av-char { display: inline-flex; align-items: center; gap: 4px; }
        .av-char img { width: 20px; height: 20px; border-radius: 50%; }
      `,
      editor: {
        async completions(partial) {
          // Search anthroverse API for matching character names
          return [{ label: "AzaelDragon", detail: "Dragon · Artist" }];
        },
        async hover(node) {
          return `**${node.content}**\nSpecies: ${node.attributes.species}`;
        }
      }
    },

    "anthroverse-commission": {
      type: "block",
      async render(node, ctx) {
        return `<div class="commission-sheet">...</div>`;
      },
      styles: `...`
    }
  },

  themes: {
    "anthroverse-profile": {
      name: "anthroverse-profile",
      variables: {
        "--etch-bg": "#1a1525",
        "--etch-accent": "#9b59b6"
      },
      css: `.av-char { border-color: var(--etch-accent); }`
    }
  }
} satisfies EtchPlugin;
```

---

## 4. Plugin Distribution & CLI Tooling

### 4.1 Package Convention

Plugins are published to npm under the `etch-plugin-` prefix:

- Plugin name: `anthroverse`
- npm package: `etch-plugin-anthroverse`
- The `package.json` must include an `"etch-plugin"` field (for discovery/validation).

### 4.2 CLI Commands

```
etch add <name>              Install a plugin to the current project
etch add <name> --global     Install a plugin globally
etch remove <name>           Remove a project-local plugin
etch plugins                 List installed plugins (project + global)
```

**`etch add <name>`:**

1. Resolves `etch-plugin-<name>` from the npm registry.
2. Downloads the package into `.etch/plugins/etch-plugin-<name>/`.
3. Adds `<name>` to the `plugins` array in `etch.config.json` (creates the file if it doesn't exist).

**`etch add <name> --global`:**

1. Same resolution and download, but installs to `~/.etch/plugins/`.
2. Does not modify project config.

**`etch remove <name>`:**

1. Removes the plugin directory from `.etch/plugins/`.
2. Removes the entry from `etch.config.json`.
3. Leaves any global install untouched.

**`etch plugins`:**

Lists all resolved plugins with source location:

```
Plugins:
  ● math-extended    2.1.0   (project)
  ● diagrams         0.4.2   (global)
```

### 4.3 Storage Layout

```
~/.etch/
  plugins/                      # Global plugins
    etch-plugin-diagrams/
      package.json
      index.js

project/
  .etch/
    plugins/                    # Project-local plugins
      etch-plugin-math-extended/
        package.json
        index.js
      etch-plugin-anthroverse/
        package.json
        index.js
  etch.config.json
```

### 4.4 Resolution Order

When building the plugin registry, the runtime resolves in this order:

1. **Project-local:** `.etch/plugins/` in the working directory.
2. **Global:** `~/.etch/plugins/`.
3. **Config-declared:** plugins listed in `etch.config.json` must resolve from one of the above locations.
4. **Frontmatter:** per-file `plugins` field merges additively with the project config.

If a plugin is declared in config or frontmatter but not installed, the renderer emits a warning (not an error) and renders a fallback placeholder.

**Effective plugin list.** The effective plugin list for a given file render is the **union** of plugins declared in `etch.config.json` and plugins declared in that file's frontmatter (deduplicated by name). For each plugin in the effective list, the runtime resolves the module by checking project-local first (`.etch/plugins/`), then global (`~/.etch/plugins/`). If a plugin in the effective list cannot be resolved from either location, it is treated as missing (warning + fallback).

---

## 5. Configuration

### 5.1 Project Configuration: `etch.config.json`

```json
{
  "plugins": [
    "math-extended",
    {
      "name": "anthroverse",
      "config": {
        "apiBase": "https://api.anthroverse.com",
        "defaultSpecies": "dragon"
      }
    }
  ],
  "theme": "default"
}
```

- **`plugins`** — array of plugin names (strings) or objects with `name` and `config`.
- **`theme`** — active theme name. Can be a built-in theme or a plugin-provided theme.

### 5.2 Per-File Frontmatter

```yaml
---
plugins:
  - math-extended
  - anthroverse:
      showNsfw: true
theme: commission-dark
---
```

- Frontmatter `plugins` **merges additively** with project config (does not replace). Plugins are deduplicated by name.
- Plugin-specific config in frontmatter **fully replaces** the project-level config object for that plugin in that file. It is not a shallow merge — if frontmatter provides config for plugin X, the entire config object for X comes from frontmatter, not a merge of project + frontmatter fields.
- Frontmatter `theme` overrides the project theme for that file only.

---

## 6. Theme System

### 6.1 Theme Anatomy

```typescript
interface EtchTheme {
  /** Theme name. */
  name: string;

  /** CSS custom properties applied to the document root. */
  variables: {
    "--etch-bg": string;
    "--etch-text": string;
    "--etch-heading-font": string;
    "--etch-body-font": string;
    "--etch-accent": string;
    "--etch-code-bg": string;
    // Extensible — plugins may define additional variables
    [key: `--etch-${string}`]: string;
  };

  /** Additional CSS rules beyond variables. */
  css?: string;

  /** Dark mode variant, auto-applied via prefers-color-scheme. */
  darkMode?: {
    variables: Record<string, string>;
    css?: string;
  };
}
```

### 6.2 Theme Sources

Themes can come from two places:

1. **Plugin-provided** — themes shipped in a plugin's `themes` field.
2. **Built-in** — themes shipped with Etch:
   - `default` — clean serif (current Georgia-based theme).
   - `minimal` — sans-serif, compact.
   - `academic` — LaTeX-like appearance.

If a plugin theme and a built-in theme share a name, the plugin theme wins. Unknown theme names fall back to `default`.

### 6.3 How Themes Apply

- Core elements and plugin output both read from `--etch-*` CSS variables.
- Plugins use their own class prefixes for structural CSS (layout, spacing) but reference `--etch-*` variables for colors and fonts.
- Switching themes re-skins everything — core and plugin output — without plugin-specific theme work.
- Dark mode is first-class: every theme can declare a `darkMode` variant, auto-applied via `prefers-color-scheme` media query.
- IDE previews map the editor's theme (light/dark) to Etch theme variables.

---

## 7. IDE Integration

### 7.1 Plugin Lifecycle in Editors

When an editor with Etch support opens a workspace:

1. **Activate** — initialize WASM parser, create diagnostic collection.
2. **Discover** — read `etch.config.json`, scan `.etch/plugins/` and `~/.etch/plugins/`.
3. **Load** — import each plugin module, call `setup(ctx)`, register directive handlers in the render pipeline, collect styles, register editor support providers.
4. **Ready** — listen for document edits.

On `etch.config.json` change: reload the plugin registry and re-render active previews.

### 7.2 Preview Rendering Pipeline

On every document edit (debounced):

1. **Parse** (WASM) — `etch_wasm.parse(source)` → AST + diagnostics. Core math directives produce MathML. Unknown directives remain directive AST nodes.
2. **Core render** (WASM) — `etch_wasm.render_html(source)` → HTML with MathML for math, placeholder elements for plugin directives (`<div data-etch-directive="name" data-etch-kind="block" data-etch-id="1" ...>` or `<span ...>` for inline directives).
3. **Plugin pipeline** (JS) — walk placeholder elements, find matching plugin handler, build a `DirectiveNode` from the parsed AST via directive id lookup, call `handler.render(node, ctx)`, replace placeholder with output. If no handler: render fallback warning.
4. **Assemble** — inject theme CSS variables, inject collected plugin styles, wrap in preview shell (CSP, fonts), post to webview.

### 7.3 Editor Intelligence

Plugins providing `EditorSupport` enhance the IDE experience:

- **Autocomplete** — when the user types `:directive-name[`, the extension calls the plugin's `completions()` with the partial text and shows suggestions.
- **Hover** — when the user hovers over a directive instance, the extension calls `hover()` and displays the result in a tooltip.
- **Diagnostics** — plugins report warnings/errors via `ctx.warn()` and `ctx.error()` during rendering. These appear as squiggly underlines in the editor and entries in the Problems panel.

### 7.4 Missing Plugin UX

When a document references a directive with no installed handler:

- **Preview:** renders a visible warning box: `Unknown directive: <name>`.
- **Diagnostics:** emits an information-level diagnostic on the directive's source location.
- **No silent failures.** The document always renders — unhandled directives degrade to visible, helpful placeholders.

### 7.5 Cross-Editor Portability

The render pipeline lives in `@etch-markup/etch-plugin-pipeline` and is pure JS — not VS Code-specific. Any editor integration follows the same pattern:

1. Read config, discover plugins from `.etch/plugins/` and `~/.etch/plugins/`.
2. Load plugin modules, call `setup()`.
3. Run the render pipeline (parse via WASM, core render, plugin transform).
4. Display the output.

`EditorSupport` (completions, hover) is optional — plugins work without it. Advanced editor features degrade gracefully in simpler editors.

---

## 8. Plugin SDK

The `@etch-markup/etch-plugin-sdk` package provides:

- **TypeScript types** — `AstNode`, `Document`, `DirectiveSpan`, `DirectiveNode`, `RenderContext`, `EditorSupport`, `EtchTheme`, `Completion`, `PluginContext`, `DirectiveHandler`, `EtchPlugin`.
- **Utility functions** — helpers for common tasks (`escapeHtml`, `parseAttributes`).
- **No runtime dependency** — the SDK is types and utilities only. Plugins do not bundle the SDK at runtime.

Plugin authors install it as a dev dependency:

```
npm install --save-dev @etch-markup/etch-plugin-sdk
```

---

## 9. Future Considerations

### 9.1 WYSIWYG Editor (Tier 3)

The planned WYSIWYG editor will surface the same plugin system through a GUI:

- A plugin browser where users click to enable/disable plugins.
- Behind the scenes, the editor calls `etch add`/`etch remove` or manages plugins directly.
- Non-technical users never interact with CLI or npm.

### 9.2 Custom Plugin Registry

If the ecosystem grows sufficiently, a dedicated Etch plugin registry (similar to Deno's JSR) could provide:

- Curated discovery and search.
- Quality/security review for listed plugins.
- A web-based plugin browser.

The plugin format (JS modules with `EtchPlugin` interface) remains the same — only the distribution channel changes. Migration from npm to a custom registry is a packaging change, not a rewrite.

### 9.3 First-Party Plugins

Planned first-party plugins maintained by the Etch team:

- **`math-extended`** — matrices, aligned equations, piecewise functions, and other advanced LaTeX math constructs.
- **`diagrams`** — flowcharts, sequence diagrams, and other visual constructs.
- Additional plugins as community needs emerge.
