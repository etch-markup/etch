import type { EtchTheme } from "@etch-markup/etch-plugin-sdk";

export const BUILTIN_THEMES: Record<string, EtchTheme> = {
  default: {
    name: "default",
    variables: {
      "--etch-bg": "#ffffff",
      "--etch-text": "#1a1a1a",
      "--etch-heading-font": "Georgia, serif",
      "--etch-body-font": "Georgia, serif",
      "--etch-accent": "#2563eb",
      "--etch-code-bg": "#f5f5f5"
    },
    darkMode: {
      variables: {
        "--etch-bg": "#1a1a1a",
        "--etch-text": "#e0e0e0",
        "--etch-code-bg": "#2a2a2a"
      }
    }
  },
  minimal: {
    name: "minimal",
    variables: {
      "--etch-bg": "#ffffff",
      "--etch-text": "#333333",
      "--etch-heading-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-body-font":
        "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
      "--etch-accent": "#0066cc",
      "--etch-code-bg": "#f0f0f0"
    },
    darkMode: {
      variables: {
        "--etch-bg": "#1e1e1e",
        "--etch-text": "#d4d4d4",
        "--etch-code-bg": "#2d2d2d"
      }
    }
  },
  academic: {
    name: "academic",
    variables: {
      "--etch-bg": "#fffff8",
      "--etch-text": "#111111",
      "--etch-heading-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-body-font":
        "'Computer Modern Serif', 'Latin Modern Roman', Georgia, serif",
      "--etch-accent": "#8b0000",
      "--etch-code-bg": "#f5f2eb"
    },
    darkMode: {
      variables: {
        "--etch-bg": "#1a1a18",
        "--etch-text": "#d4d0c8",
        "--etch-code-bg": "#2a2820"
      }
    }
  }
};

export function assembleThemeCSS(theme: EtchTheme): string {
  let css = ":root {\n";
  for (const [key, value] of Object.entries(theme.variables)) {
    css += `  ${key}: ${value};\n`;
  }
  css += "}\n";

  if (theme.css) {
    css += `${theme.css}\n`;
  }

  if (theme.darkMode) {
    css += "@media (prefers-color-scheme: dark) {\n  :root {\n";
    for (const [key, value] of Object.entries(theme.darkMode.variables)) {
      css += `    ${key}: ${value};\n`;
    }
    css += "  }\n";
    if (theme.darkMode.css) {
      css += `${theme.darkMode.css}\n`;
    }
    css += "}\n";
  }

  return css;
}
