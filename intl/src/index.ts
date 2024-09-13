import * as React from 'react';
import { RichTextNode } from './formatters';

export * from './formatters';
export { IntlManager, DEFAULT_LOCALE, type FormatFunction } from './intl-manager';
export {
  createLoader,
  loadAllMessagesInLocale,
  waitForAllDefaultIntlMessagesLoaded,
} from './message-loader';
export type * from './types.d.ts';

/**
 * A type representing any message that has been formatted using one of the
 * default formatting methods in the `@discord/intl` system. Use this type
 * when you don't care about whether a message is static or contains rich text
 * or has dynamically-formatted values, when you need to handle both in tandem,
 * or when you need a generic return type for a function returning "any
 * any-already formatted message".
 *
 * Generally, this is like `React.ReactNode`, accepting both strings and rich
 * `ReactElement`s, but it is more specific to disallow nullish values and
 * other special React elements like `ReactPortal`. Seeing this type, you can
 * be confident that the value is intended to be the result of formatting an
 * intl message, even if the actual value comes from elsewhere (like a
 * user-generated string).
 */
export type ReactIntlMessage = string | React.ReactElement;

/**
 * The return value of `formatToParts` from `@discord/intl`, this type
 * represents any AST structure for a message rendered using this system.
 * The AST generally follows a Markdown-like structure, with text nodes
 * interspersed within and around rich text formatting nodes.
 *
 * ASTs are created _after_ all placeholders have been filled by values
 * from a call to `formatToParts`, meaning they are intended to be fully static
 * structures passed around for custom rendering functions to use.
 */
export type IntlMessageAst = RichTextNode;
