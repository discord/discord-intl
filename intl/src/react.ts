import * as React from 'react';

import { IntlManager } from './intl-manager';

import type { FormatValuesFor, RichTextElementMap, TypedIntlMessageGetter } from './types';

export class IntlManagerReact<
  DefaultElements extends RichTextElementMap,
  DefaultValues extends keyof DefaultElements,
> extends IntlManager<DefaultElements, DefaultValues> {
  /**
   * A reactive component form of `intl.format` that automatically updates when
   * the application's locale changes and when new data is loaded for the
   * subject message.
   */
  IntlMessage = <T extends TypedIntlMessageGetter<object | undefined>>(props: {
    message: T;
    values?: Omit<FormatValuesFor<T>, DefaultValues>;
  }) => {
    const { message, values } = props;
    // Use the locale from this point in the application, which may be
    // different from the global locale.
    const locale = React.useSyncExternalStore(this.onLocaleChange, () => this.currentLocale);
    // Source the actual message to render for that locale from its loader.
    // TODO(faulty): This can and should be replaced by a
    // `use(messagesLoadedPromise)` once `use` is shipped to stable.
    React.useSyncExternalStore(message.onChange, () => message(locale));
    // The return the formatted version of that string.
    if (typeof message === 'string') return message;

    return React.createElement(
      React.Fragment,
      undefined,
      this.formatToParts(message, values as Omit<FormatValuesFor<T>, DefaultValues> | never),
    );
  };

  /**
   * Format the given message as a React component, allowing it to listen for
   * and respond to updates about the current locale and other relevant
   * information.
   */
  format<T extends TypedIntlMessageGetter<object | undefined>>(message: T): JSX.Element;
  format<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values: Omit<FormatValuesFor<T>, DefaultValues>,
  ): JSX.Element;
  format(message: string): JSX.Element;
  format<T extends string | TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values?: Omit<FormatValuesFor<T>, DefaultValues>,
  ) {
    // A string literal means there's no locale dependency, so it can just be
    // formatted directly without any subscription.
    if (typeof message === 'string') return message;

    return React.createElement(this.IntlMessage, { message, values });
  }
}
