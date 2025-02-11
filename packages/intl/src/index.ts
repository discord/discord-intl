export { type DataFormatters, makeDataFormatters } from './data-formatters';
export { FormatBuilder, FormatBuilderConstructor, bindFormatValues } from './format';
export * from './formatters';
export { runtimeHashMessageKey } from './hash';
export { IntlManager, DEFAULT_LOCALE, type FormatFunction } from './intl-manager';
export {
  createLoader,
  loadAllMessagesInLocale,
  waitForAllDefaultIntlMessagesLoaded,
  MessageLoader,
} from './message-loader';
export type * from './types.d.ts';

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
import { type RichTextNode } from './formatters';
export type IntlMessageAst = RichTextNode;
