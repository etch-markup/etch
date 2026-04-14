export interface FrontmatterRecord {
  [key: string]: FrontmatterValue;
}

export type FrontmatterValue =
  | string
  | number
  | boolean
  | null
  | FrontmatterValue[]
  | FrontmatterRecord;

// The Rust/WASM bridge serializes frontmatter objects as plain JS objects.
export type FrontmatterFields = FrontmatterRecord;

export interface Frontmatter {
  raw: string;
  fields: FrontmatterFields;
}

export interface SourcePosition {
  line: number;
  column: number;
}

export interface SourceSpan {
  start: SourcePosition;
  end: SourcePosition;
}

// Serialized attributes always expose key/value pairs as plain JS objects.
export type AttributePairs = Record<string, string>;

export interface Attributes {
  id?: string;
  classes?: string[];
  pairs?: AttributePairs;
}

export interface TableCell {
  content: Inline[];
}

export type Alignment = 'None' | 'Left' | 'Center' | 'Right';

export interface ListItem {
  content: Block[];
  checked?: boolean;
}

export interface DefinitionItem {
  term: Inline[];
  definitions: Block[][];
}

export interface Document {
  frontmatter?: Frontmatter;
  body: Block[];
}

export interface ParagraphBlock {
  type: 'Paragraph';
  content: Inline[];
  attrs?: Attributes;
}

export interface HeadingBlock {
  type: 'Heading';
  level: number;
  content: Inline[];
  attrs?: Attributes;
}

export interface CodeBlock {
  type: 'CodeBlock';
  language?: string;
  content: string;
  attrs?: Attributes;
}

export interface BlockQuote {
  type: 'BlockQuote';
  content: Block[];
  attrs?: Attributes;
}

export interface ListBlock {
  type: 'List';
  ordered: boolean;
  items: ListItem[];
  attrs?: Attributes;
}

export interface TableBlock {
  type: 'Table';
  headers: TableCell[];
  rows: TableCell[][];
  alignments: Alignment[];
  attrs?: Attributes;
}

export interface ThematicBreak {
  type: 'ThematicBreak';
}

export interface BlockDirective {
  type: 'BlockDirective';
  directive_id: number;
  span: SourceSpan;
  name: string;
  label?: Inline[];
  raw_label?: string;
  attrs?: Attributes;
  raw_body: string;
  body: Block[];
}

export interface ContainerDirective {
  type: 'ContainerDirective';
  directive_id: number;
  span: SourceSpan;
  name: string;
  label?: Inline[];
  raw_label?: string;
  attrs?: Attributes;
  raw_body: string;
  body: Block[];
  named_close: boolean;
}

export interface FootnoteDefinition {
  type: 'FootnoteDefinition';
  label: string;
  content: Block[];
}

export interface DefinitionList {
  type: 'DefinitionList';
  items: DefinitionItem[];
  attrs?: Attributes;
}

export type Block =
  | ParagraphBlock
  | HeadingBlock
  | CodeBlock
  | BlockQuote
  | ListBlock
  | TableBlock
  | ThematicBreak
  | BlockDirective
  | ContainerDirective
  | FootnoteDefinition
  | DefinitionList;

export interface TextInline {
  type: 'Text';
  value: string;
}

export interface Emphasis {
  type: 'Emphasis';
  content: Inline[];
}

export interface Strong {
  type: 'Strong';
  content: Inline[];
}

export interface Strikethrough {
  type: 'Strikethrough';
  content: Inline[];
}

export interface InlineCode {
  type: 'InlineCode';
  value: string;
}

export interface Superscript {
  type: 'Superscript';
  content: Inline[];
}

export interface Subscript {
  type: 'Subscript';
  content: Inline[];
}

export interface Highlight {
  type: 'Highlight';
  content: Inline[];
}

export interface Insert {
  type: 'Insert';
  content: Inline[];
}

export interface Spoiler {
  type: 'Spoiler';
  content: Inline[];
}

export interface Link {
  type: 'Link';
  url: string;
  title?: string;
  content: Inline[];
  attrs?: Attributes;
}

export interface Image {
  type: 'Image';
  url: string;
  alt: string;
  title?: string;
  attrs?: Attributes;
}

export interface AutoLink {
  type: 'AutoLink';
  url: string;
}

export interface InlineDirective {
  type: 'InlineDirective';
  directive_id: number;
  span: SourceSpan;
  name: string;
  content?: Inline[];
  raw_content?: string;
  attrs?: Attributes;
}

export interface FootnoteReference {
  type: 'FootnoteReference';
  label: string;
}

export interface SoftBreak {
  type: 'SoftBreak';
}

export interface HardBreak {
  type: 'HardBreak';
}

export type Inline =
  | TextInline
  | Emphasis
  | Strong
  | Strikethrough
  | InlineCode
  | Superscript
  | Subscript
  | Highlight
  | Insert
  | Spoiler
  | Link
  | Image
  | AutoLink
  | InlineDirective
  | FootnoteReference
  | SoftBreak
  | HardBreak;

export type ParseErrorKind = 'Error' | 'Warning';

export interface ParseError {
  kind: ParseErrorKind;
  message: string;
  line: number;
  column?: number;
}

export interface ParseResult {
  document: Document;
  errors: ParseError[];
}

export type EtchDocument = Document;
