export enum FormatJsNodeType {
  Literal = 0,
  Argument,
  Number,
  Date,
  Time,
  Select,
  Plural,
  Pound,
  Tag,
}

//#region Message AstNode
//
// These are the array-typed nodes using the (roughly) Keyless JSON format that compresses far more
// than a full object AST when serializing and bundling.
//
// Everything in the `@discord/intl` system works with this compressed AST format, but utilities are
// provided to convert between this and the FormatJS compatible version as needed.
export type LiteralNode = [string];
export type ArgumentNode = [FormatJsNodeType.Argument, string];
export type NumberNode = [FormatJsNodeType.Number, string, string | undefined];
export type DateNode = [FormatJsNodeType.Date, string, string | undefined];
export type TimeNode = [FormatJsNodeType.Time, string, string | undefined];
export type SelectNode = [FormatJsNodeType.Select, string, Record<string, AstNode[]>];
export type PluralNode = [
  FormatJsNodeType.Plural,
  string,
  Record<string, AstNode[]>,
  number,
  FormatJsPluralType,
];
export type PoundNode = [FormatJsNodeType.Pound];
export type TagNode = [FormatJsNodeType.Tag, string, AstNode[]];

export type AstNode =
  | LiteralNode
  | ArgumentNode
  | NumberNode
  | DateNode
  | TimeNode
  | SelectNode
  | PluralNode
  | PoundNode
  | TagNode;

export enum AstNodeIndices {
  Type = 0,
  Value = 1,
  Style = 2,
  Options = 2,
  Offset = 3,
  PluralType = 4,
  Children = 2,
}

//#region Full FormatJS Node types
//
// These are complete, strongly-typed Object nodes that match FormatJS' AST as closely as possible.

export interface FullFormatJsLiteral {
  type: FormatJsNodeType.Literal;
  value: string;
}

export interface FullFormatJsArgument {
  type: FormatJsNodeType.Argument;
  value: string;
}

export interface FullFormatJsNumber {
  type: FormatJsNodeType.Number;
  value: string;
  style?: string;
}

export interface FullFormatJsDate {
  type: FormatJsNodeType.Date;
  value: string;
  style?: string;
}

export interface FullFormatJsTime {
  type: FormatJsNodeType.Time;
  value: string;
  style?: string;
}

export interface FullFormatJsSelect {
  type: FormatJsNodeType.Select;
  value: string;
  options: Record<string, { value: FullFormatJsNode[] }>;
}

export type FormatJsPluralType = 'cardinal' | 'ordinal';

export interface FullFormatJsPlural {
  type: FormatJsNodeType.Plural;
  value: string;
  options: Record<string, { value: FullFormatJsNode[] }>;
  offset: number;
  pluralType: FormatJsPluralType;
}

export interface FullFormatJsPound {
  type: FormatJsNodeType.Pound;
}

export interface FullFormatJsTag {
  type: FormatJsNodeType.Tag;
  value: string;
  children: FullFormatJsNode[];
}

export type FullFormatJsNode =
  | FullFormatJsLiteral
  | FullFormatJsArgument
  | FullFormatJsNumber
  | FullFormatJsDate
  | FullFormatJsTime
  | FullFormatJsSelect
  | FullFormatJsPlural
  | FullFormatJsPound
  | FullFormatJsTag;

//#endregion

//#region Hydration
//
// Utilities for converting compressed Keyless Json into the fully-typed FormatJS node structure.

// Using a static object here ensures there's only one allocation for it, which
// will likely appear in many, many messages. Freezing ensures that the shared
// object can't be accidentally modified if a consumer tries to transform the
// AST.
export const FORMAT_JS_POUND: FullFormatJsPound = Object.freeze({ type: 7 });

function hydrateArray(elements: Array<Array<any>>) {
  for (let i = 0; i < elements.length; i++) {
    elements[i] = hydrateFormatJsAst(elements[i]);
  }
}

function hydratePlural(keyless: Array<any>): FullFormatJsNode {
  const [type, value, options, offset, pluralType] = keyless;
  // This tries to be efficient about updating the value for each option
  // with a parsed version, reusing the same receiving object, and even
  // the inner key within it, just replacing the end value with the
  // parsed version.
  // This saves multiple allocations compared to building a new object
  // from scratch, either for the whole options object or for each value.
  for (const key in options) {
    hydrateArray(options[key].value);
  }
  const valueOptions = options.map((option) => ({ value: option }));
  // `pluralType` is technically only valid on `Plural` nodes, even
  // though the structure is identical to `Select`.
  if (type === FormatJsNodeType.Plural) {
    return {
      type,
      value,
      options,
      offset: valueOptions,
      pluralType,
    };
  } else {
    return { type, value, options: valueOptions, offset };
  }
}

function hydrateSingle(keyless: Array<any>): FullFormatJsNode {
  const [type] = keyless;
  switch (type) {
    case FormatJsNodeType.Literal:
      return { type: 0, value: keyless[0] };
    case FormatJsNodeType.Argument:
      return { type, value: keyless[1] };
    case FormatJsNodeType.Number:
    case FormatJsNodeType.Date:
    case FormatJsNodeType.Time:
      return { type, value: keyless[1], style: keyless[2] };
    case FormatJsNodeType.Select:
    case FormatJsNodeType.Plural:
      return hydratePlural(keyless);
    case FormatJsNodeType.Pound:
      return FORMAT_JS_POUND;
    case FormatJsNodeType.Tag: {
      const [type, value, children] = keyless;
      hydrateArray(children);
      return { type, value, children: children };
    }
    default:
      throw new Error(`FormatJS keyless JSON encountered an unknown type: ${type}`);
  }
}

/**
 * Hydrate the given keyless JSON into a FormatJS-compatible AST structure.
 * The given object is reused as much as possible, meaning it cannot be safely
 * used again after hydrating.
 */
export function hydrateFormatJsAst(keyless: Array<Array<any>>): FullFormatJsNode[];
export function hydrateFormatJsAst(keyless: Array<any>): FullFormatJsNode | FullFormatJsNode[] {
  // If the first element of the array is itself an array, then we have a list
  // of elements to parse rather than a single object.
  if (Array.isArray(keyless[0])) {
    hydrateArray(keyless);
    return keyless;
  } else if (keyless.length === 0) {
    // Some entries can be empty arrays, like an empty set of children, and those can
    // be preserved.
    return keyless;
  }

  return hydrateSingle(keyless);
}

//#endregion

//#region Compression
//
// Utilities for converting fully-typed FormatJS-compatible nodes into the array-typed structure
// used internally.
//
// In normal operation, messages are compiled to the array format ahead of time and these
// conversions are skipped entirely at runtime. But they are provided for compatibility to allow
// either keyless or full JSON messages to be used interchangeably with a low conversion cost.

export function compressFormatJsToAst(node: FullFormatJsNode): AstNode;
export function compressFormatJsToAst(node: FullFormatJsNode[]): AstNode[];
export function compressFormatJsToAst(
  node: FullFormatJsNode | FullFormatJsNode[],
): AstNode | AstNode[] {
  if (Array.isArray(node)) {
    return node.map((element) => compressFormatJsToAst(element));
  }

  switch (node.type) {
    case FormatJsNodeType.Literal:
      return [node.value];
    case FormatJsNodeType.Argument:
      return [node.type, node.value];
    case FormatJsNodeType.Number:
    case FormatJsNodeType.Date:
    case FormatJsNodeType.Time:
      return [node.type, node.value, node.style];
    case FormatJsNodeType.Select: {
      const reducedOptions: Record<string, AstNode[]> = {};
      for (const [name, option] of Object.entries(node.options)) {
        reducedOptions[name] = compressFormatJsToAst(option.value);
      }
      return [node.type, node.value, reducedOptions];
    }
    case FormatJsNodeType.Plural: {
      const reducedOptions: Record<string, AstNode[]> = {};
      for (const [name, option] of Object.entries(node.options)) {
        reducedOptions[name] = compressFormatJsToAst(option.value);
      }
      return [node.type, node.value, reducedOptions, node.offset, node.pluralType];
    }
    case FormatJsNodeType.Pound:
      return [node.type];
    case FormatJsNodeType.Tag:
      return [node.type, node.value, compressFormatJsToAst(node.children)];
  }
}

/**
 * Returns true if the given `node` is a compressed `AstNode` rather than a `FullFormatJsNode`. This
 * also works for checking arrays of nodes, returning true if the nodes of the array are compressed.
 */
export function isCompressedAst(node: AstNode | FullFormatJsNode): node is AstNode;
export function isCompressedAst(node: AstNode[] | FullFormatJsNode[]): node is AstNode[];
export function isCompressedAst(
  node: AstNode | FullFormatJsNode | AstNode[] | FullFormatJsNode[],
): boolean {
  // Not an array at all means this is a singular fully-typed node.
  if (!Array.isArray(node)) return false;
  // Otherwise just check the first element. If it's an array, then the ast is compressed as
  // keyless.
  return Array.isArray(node[0]);
}
