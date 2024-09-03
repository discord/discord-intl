import * as React from 'react';

import { IntlManager } from './intl-manager';

import type { RequiredFormatValues, RichTextElementMap, TypedIntlMessageGetter } from './types';

type ReactHandlerEvent = React.MouseEvent | React.KeyboardEvent;
type ReactClickHandler = (e: ReactHandlerEvent) => void;

type ReactFunctionTypes = {
  // TODO: Allowing `undefined` on these is a migration artifact that should be
  // removed. Functions, like all other values, should be required when
  // formatting to avoid unexpected missing behavior.
  link:
    | undefined
    | ((content: React.ReactNode | React.ReactNode[], key: string) => React.ReactNode);

  hook:
    | undefined
    | ((content: React.ReactNode | React.ReactNode[], key: string) => React.ReactNode);
  handler:
    | undefined
    | ReactClickHandler
    | { onClick: ReactClickHandler }
    | { onContextMenu: ReactClickHandler }
    | { onClick: ReactClickHandler; onContextMenu: ReactClickHandler };
};

export class IntlManagerReact<DefaultElements extends RichTextElementMap> extends IntlManager<
  DefaultElements,
  ReactFunctionTypes
> {
  /**
   * A reactive component form of `intl.format` that automatically updates when
   * the application's locale changes and when new data is loaded for the
   * subject message.
   */
  IntlMessage = <T extends TypedIntlMessageGetter<object | undefined>>(props: {
    message: T;
    values?: RequiredFormatValues<T, DefaultElements, ReactFunctionTypes> | never;
  }) => {
    const { message, values } = props;
    // Use the locale from this point in the application, which may be
    // different from the global locale.
    const locale = React.useSyncExternalStore(this.onLocaleChange, () => this.currentLocale);
    // TODO(faulty): This can and should be replaced with
    // `use(messagesLoadedPromise)` once `use` is shipped to stable.
    // Source the actual message to render for that locale from its loader.
    React.useSyncExternalStore(message.onChange, () => message(locale));
    // If there are no object parts in the message, it has no formatting and can just be returned as
    // a plain string.
    if (typeof message === 'string') return message;

    const parts = this.formatToParts(message, values);
    if (parts.length === 1 && typeof parts[0] === 'string') return parts[0];
    return React.createElement(React.Fragment, undefined, this.formatToParts(message, values));
  };

  /**
   * Format the given message as a React component, allowing it to listen for
   * and respond to updates about the current locale and other relevant
   * information.
   */
  format<T extends TypedIntlMessageGetter<object | undefined>>(message: T): React.ReactElement;
  format<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values: RequiredFormatValues<T, DefaultElements, ReactFunctionTypes>,
  ): React.ReactElement;
  format<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values?: RequiredFormatValues<T, DefaultElements, ReactFunctionTypes>,
  ): React.ReactElement {
    return React.createElement(this.IntlMessage<T>, { message, values });
  }
}
