import {
  TYPE as FormatElementType,
  parse as parseMessageFormat,
} from '@formatjs/icu-messageformat-parser';
import {
  Formats,
  Formatters,
  MessageFormatPart,
  formatToParts as libFormatToParts,
} from 'intl-messageformat';

import type { MessageFormatElement } from '@formatjs/icu-messageformat-parser';

export class InternalIntlMessage {
  locale: string;
  message?: string;
  ast: MessageFormatElement[];
  /**
   * A stripped-down representation of the message with no rich formatting
   * elements preserved. This will be lazily created the first time it is
   * requested (e.g., by calling `formatToPlainString`).
   */
  plainAst?: MessageFormatElement[];

  constructor(messageOrAst: string | MessageFormatElement[], locale: string) {
    this.locale = locale;

    if (typeof messageOrAst === 'string') {
      // TODO: This part should pre-parse our Markdown shorthand, too, so that
      // the AST that comes out of this is a standardized format, and we don't
      // have to do any special handling after calling `format`. That also
      // removes the need for handling "unsafe" values, since they'll be
      // parsed before values get interpolated. We can actually do that in the
      // loader, too, and avoid any runtime cost for it.

      this.message = messageOrAst;
      this.ast = parseMessageFormat(messageOrAst);
    } else {
      this.ast = messageOrAst;
    }

    this.plainAst = undefined;
  }

  /**
   * Format this message with the given locale into an AST of chunks that can
   * then be used or rendered into strings, React elements, or anything else as
   * needed.
   */
  formatToParts<T>(
    formatters: Formatters,
    formats: Formats,
    values?: Record<string, any>,
  ): Array<MessageFormatPart<T>> {
    return libFormatToParts(this.ast, this.locale, formatters, formats, values);
  }

  /**
   * Formats this message just like `formatToParts`, but strips all stylistic
   * elements and tags and returns a plain string with only the literal text of
   * the message remaining.
   *
   * This method works by quickly scanning the string and removing any tag
   * elements, formatting, then concatenating the pieces into a single result.
   *
   * Note that this function does _not_ handle serializing values other than
   * plain/primitive values into strings. All values are simply converted using
   * the `String()` constructor. So, for example, passing a function as a value
   * will directly serialize the function definition into a string, rather than
   * calling the function or evaluating the result.
   */
  formatToPlainString(
    formatters: Formatters,
    formats: Formats,
    values?: Record<string, any>,
  ): string {
    // Lazily create the reduced AST if it hasn't already been done.
    if (this.plainAst == null) {
      this.plainAst = [];
      for (const part of this.ast) {
        this.plainAst.push(...this._removeRichTags(part));
      }
    }

    const parts = libFormatToParts(this.plainAst, this.locale, formatters, formats, values);
    let result = '';
    for (const part of parts) {
      result += String(part.value);
    }

    return result;
  }

  /**
   * Returns the same element with any formatting tags removed. If the element
   * itself is a tag, the children will be hoisted and returned, otherwise the
   * element is returned as an array of itself.
   */
  _removeRichTags(element: MessageFormatElement): MessageFormatElement[] {
    if (element.type === FormatElementType.tag) {
      const childContent = [];
      for (const child of element.children) {
        childContent.push(...this._removeRichTags(child));
      }
      return childContent;
    } else {
      return [element];
    }
  }
}
