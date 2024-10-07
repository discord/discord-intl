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

export interface FormatJsLiteral {
  type: FormatJsNodeType.Literal;
  value: string;
}

export interface FormatJsArgument {
  type: FormatJsNodeType.Argument;
  value: string;
}

export interface FormatJsNumber {
  type: FormatJsNodeType.Number;
  value: string;
  style?: string;
}

export interface FormatJsDate {
  type: FormatJsNodeType.Date;
  value: string;
  style?: string;
}

export interface FormatJsTime {
  type: FormatJsNodeType.Time;
  value: string;
  style?: string;
}

export interface FormatJsSelect {
  type: FormatJsNodeType.Select;
  value: string;
  options: Record<string, { value: FormatJsNode[] }>;
}

export type FormatJsPluralType = 'cardinal' | 'ordinal';

export interface FormatJsPlural {
  type: FormatJsNodeType.Plural;
  value: string;
  options: Record<string, { value: FormatJsNode[] }>;
  offset: number;
  pluralType: FormatJsPluralType;
}

export interface FormatJsPound {
  type: FormatJsNodeType.Pound;
}

export interface FormatJsTag {
  type: FormatJsNodeType.Tag;
  value: string;
  children: FormatJsNode[];
}

export type FormatJsNode =
  | FormatJsLiteral
  | FormatJsArgument
  | FormatJsNumber
  | FormatJsDate
  | FormatJsTime
  | FormatJsSelect
  | FormatJsPlural
  | FormatJsPound
  | FormatJsTag;

// Using a static object here ensures there's only one allocation for it, which
// will likely appear in many, many messages. Freezing ensures that the shared
// object can't be accidentally modified if a consumer tries to transform the
// AST.
export const FORMAT_JS_POUND: FormatJsPound = Object.freeze({ type: 7 });

function hydrateArray(elements: Array<Array<any>>) {
  for (let i = 0; i < elements.length; i++) {
    elements[i] = hydrateFormatJsAst(elements[i]);
  }
}

function hydratePlural(keyless: Array<any>): FormatJsNode {
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
  // `pluralType` is technically only valid on `Plural` nodes, even
  // though the structure is identical to `Select`.
  if (type === FormatJsNodeType.Plural) {
    return { type, value, options, offset, pluralType };
  } else {
    return { type, value, options, offset };
  }
}

function hydrateSingle(keyless: Array<any>): FormatJsNode {
  const [type] = keyless;
  switch (type) {
    case FormatJsNodeType.Literal:
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
export function hydrateFormatJsAst(keyless: Array<Array<any>>): FormatJsNode[];
export function hydrateFormatJsAst(keyless: Array<any>): FormatJsNode | FormatJsNode[] {
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

export function hydrateMessages(messages: Record<string, any>) {
  for (const key in messages) {
    messages[key] = hydrateFormatJsAst(messages[key]);
  }
  return messages;
}
