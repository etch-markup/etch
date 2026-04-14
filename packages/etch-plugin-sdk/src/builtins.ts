// Reserved directive names handled directly by the core renderer.
// Plugins must not register handlers for these names.
export const RESERVED_BUILTIN_DIRECTIVE_NAMES = [
  "abbr",
  "aside",
  "chapter",
  "cite",
  "column",
  "columns",
  "details",
  "figure",
  "kbd",
  "math",
  "note",
  "pagebreak",
  "section",
  "toc"
] as const;

export type ReservedBuiltinDirectiveName =
  (typeof RESERVED_BUILTIN_DIRECTIVE_NAMES)[number];

const RESERVED_BUILTIN_DIRECTIVE_NAME_SET = new Set<string>(
  RESERVED_BUILTIN_DIRECTIVE_NAMES
);

export function isReservedBuiltinDirectiveName(name: string): boolean {
  return RESERVED_BUILTIN_DIRECTIVE_NAME_SET.has(name);
}
