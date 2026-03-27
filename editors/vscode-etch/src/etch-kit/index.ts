import type { EtchDocument, ParseResult } from './types.js';
import {
  initializeWasm,
  parseResultFromWasm,
  parseToJsonFromWasm,
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

export function parse(input: string): EtchDocument {
  return parseWithErrors(input).document;
}

export function parseWithErrors(input: string): ParseResult {
  return parseResultFromWasm(input);
}

export function renderHtml(input: string): string {
  return renderHtmlFromWasm(input);
}

export function renderStandalone(input: string): string {
  return renderStandaloneFromWasm(input);
}

export function parseToJson(input: string): string {
  return parseToJsonFromWasm(input);
}
