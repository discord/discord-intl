/**
 * Format the given message with the provided values, removing any styling
 * and non-textual content from the message, returning a plain string.
 */
import { AnyIntlMessage, FormatterImplementation, FunctionTypes, RichTextTagNames } from '../types';
import { FormatBuilder } from '../format';
import { IntlManager } from '../intl-manager';

/**
 * Types for formatting functions when calling `bindFormatValues`, ensuring the
 * functions always yield plain strings.
 */
export type StringFunctionTypes = FunctionTypes<string>;

export class StringBuilder extends FormatBuilder<string> {
  result: string = '';

  pushRichTextTag(_tag: RichTextTagNames, children: string[], _control: string[]) {
    // Plain string formatting ignores rich text tags and just takes the
    // visible content from the children. This means the control element is not
    // important for string rendering, so the result is always just the
    // children joined together.
    for (const child of children) {
      this.result += child;
    }
  }

  pushLiteralText(text: string) {
    this.result += text;
  }

  pushObject(value: object) {
    // Objects are only included in the result if they specify a toString value directly.
    // Otherwise, they would be rendered as `[object Object]`, which is never helpful.
    if (value != null && 'toString' in value) {
      this.result += value.toString();
    }
  }

  finish(): string[] {
    return [this.result];
  }
}

export function formatToPlainString(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
): string {
  if (typeof message === 'string') return message;

  const result = this.bindFormatValues(StringBuilder, message, values);
  // StringBuilder always creates a single element array with the string value.
  return result[0];
}

export const stringFormatter: FormatterImplementation<StringFunctionTypes, string> = {
  format: formatToPlainString,
  builder: StringBuilder,
};
