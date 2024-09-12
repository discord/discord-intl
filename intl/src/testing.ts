import { IntlManager } from './intl-manager';
import {
  DEFAULT_REACT_RICH_TEXT_ELEMENTS,
  makeReactFormatter,
  stringFormatter,
  astFormatter,
  markdownFormatter,
} from './formatters';
import { TypedIntlMessageGetter } from './types';
import { InternalIntlMessage } from './message';

const intl = new IntlManager('en-US').withFormatters({
  format: makeReactFormatter(DEFAULT_REACT_RICH_TEXT_ELEMENTS),
  formatToPlainString: stringFormatter,
  formatToMarkdownString: markdownFormatter,
  formatToParts: astFormatter,
});

let message = ((locale: string) =>
  new InternalIntlMessage('hi', locale)) as TypedIntlMessageGetter<{
  locate: string;
}>;

intl.formatToMarkdownString(message, { locate: '' });
intl.formatToPlainString(message, { locate: '' });
intl.formatToParts(message, { locate: '' });
intl.format(message, { locate: '' });
intl.string(message);

type Pretty<T> = { [K in keyof T]: T[K] };

type T = Pretty<typeof message>;
