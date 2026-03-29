import type { EtchDocument, ParseResult } from './types.js';
import {
  DEFAULT_STANDALONE_STYLES,
  initializeWasm,
  parseResultFromWasm,
  parseToJsonFromWasm,
  renderDocumentFromWasm,
  renderHtmlFromWasm,
  renderStandaloneFromWasm,
} from './wasm.js';

export type {
  Alignment,
  Attributes,
  AutoLink,
  Block,
  BlockDirective,
  BlockQuote,
  CodeBlock,
  ContainerDirective,
  DefinitionItem,
  DefinitionList,
  Document,
  Emphasis,
  EtchDocument,
  FootnoteDefinition,
  FootnoteReference,
  Frontmatter,
  FrontmatterValue,
  HardBreak,
  HeadingBlock,
  Highlight,
  Image,
  Inline,
  InlineCode,
  InlineDirective,
  Insert,
  Link,
  ListBlock,
  ListItem,
  ParagraphBlock,
  ParseError,
  ParseErrorKind,
  ParseResult,
  SoftBreak,
  Strikethrough,
  Strong,
  Subscript,
  Superscript,
  TableBlock,
  TableCell,
  TextInline,
  ThematicBreak,
} from './types.js';

export async function initialize(): Promise<void> {
  await initializeWasm();
}

export { DEFAULT_STANDALONE_STYLES };

export function parse(input: string): EtchDocument {
  return parseWithErrors(input).document;
}

export function parseWithErrors(input: string): ParseResult {
  return parseResultFromWasm(input);
}

export function renderHtml(input: string): string {
  return renderHtmlFromWasm(input);
}

export function renderDocument(input: string): string {
  return renderDocumentFromWasm(input);
}

export function renderStandalone(
  input: string,
  styles: string = DEFAULT_STANDALONE_STYLES
): string {
  return renderStandaloneFromWasm(input, styles);
}

export function parseToJson(input: string): string {
  return parseToJsonFromWasm(input);
}
