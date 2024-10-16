/**
 * Similar to `formatToPlainString`, format the given message with the provided
 * values, but convert all rich text formatting back to Markdown syntax rather
 * than rendering the actual rich content. The result is a plain string that
 * can be sent through a separate Markdown renderer to get an equivalent
 * result to formatting this message directly.
 */
import {
  AnyIntlMessage,
  FormatterImplementation,
  FunctionTypes,
  RichTextFormattingMap,
  RichTextTagNames,
} from '../types';
import { IntlManager } from '../intl-manager';
import { FormatBuilderConstructor } from '../format';
import { StringBuilder } from './string';

export type MarkdownFunctionTypes = FunctionTypes<string>;

const MARKDOWN_RICH_TEXT_ELEMENTS: RichTextFormattingMap<MarkdownFunctionTypes['hook']> = {
  $b: (content) => '**' + content.join('') + '**',
  $i: (content) => '*' + content.join('') + '*',
  $del: (content) => '~~' + content.join('') + '~~',
  $code: (content) => '`' + content.join('') + '`',
  $link: (content, _, [target]) => '[' + content.join('') + '](' + target + ')',
  $p: (content) => content.join('') + '\n\n',
};

class MarkdownBuilder extends StringBuilder {
  result: string = '';

  pushRichTextTag(tag: RichTextTagNames, children: string[], control: string[]) {
    this.result += MARKDOWN_RICH_TEXT_ELEMENTS[tag](children, '', control);
  }
}

export function formatToMarkdownString(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
  Builder: FormatBuilderConstructor<string> = MarkdownBuilder,
): string {
  if (typeof message === 'string') return message;

  const result = this.bindFormatValues(Builder, message, values);
  // MarkdownBuilder always creates a single-element array with the string value.
  return result[0];
}

export const markdownFormatter: FormatterImplementation<MarkdownFunctionTypes, string> = {
  format: formatToMarkdownString,
  builder: MarkdownBuilder,
};
