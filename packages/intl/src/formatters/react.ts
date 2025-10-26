/**
 * Format the given message as a React component, allowing it to listen for
 * and respond to updates about the current locale and other relevant
 * information.
 */
import type {
  AnyIntlMessage,
  FormatterImplementation,
  FunctionTypes,
  RichTextFormattingMap,
  RichTextTagNames
} from '../types';
import * as React from 'react';
import { IntlManager } from '../intl-manager';
import { BuilderContext, FormatBuilder, FormatBuilderConstructor } from '../format';

export type ReactHandlerEvent = React.MouseEvent | React.KeyboardEvent;
export type ReactClickHandler = (e: ReactHandlerEvent) => void;

export type ReactFunctionTypes = FunctionTypes<
  React.ReactNode,
  | ReactClickHandler
  | { onClick: ReactClickHandler }
  | { onContextMenu: ReactClickHandler }
  | { onClick: ReactClickHandler; onContextMenu: ReactClickHandler }
>;

const h = React.createElement;
export const DEFAULT_REACT_RICH_TEXT_ELEMENTS: RichTextFormattingMap<ReactFunctionTypes['hook']> = {
  $b: (content, key) => h('strong', { key }, content),
  $i: (content, key) => h('em', { key }, content),
  $del: (content, key) => h('del', { key }, content),
  $code: (content, key) => h('code', { key }, content),
  $link: (content, key, [href]) => h('a', { href, key }, content),
  $p: (content, key) => h('p', { key }, content),
};

export type ReactBuilder = {
  new (context: BuilderContext): FormatBuilder<React.ReactNode>;
};

/**
 * Creates a new `FormatBuilder` class that constructs a React element tree
 * from the message using the given `richTextElements` to apply formatting.
 * This allows consumers to inject their own design system components and
 * overrides for rendering elements like links, paragraphs, and code blocks.
 * @param richTextElements
 */
function createReactBuilder(
  richTextElements: RichTextFormattingMap<ReactFunctionTypes['hook']>,
): ReactBuilder {
  return class extends FormatBuilder<React.ReactNode> {
    _nodeKey: number = 0;
    result: React.ReactNode[] = [];

    pushRichTextTag(
      tag: RichTextTagNames,
      children: React.ReactNode[],
      control: React.ReactNode[],
    ) {
      this.result.push(
        richTextElements[tag](
          children,
          `${this.context.keyPrefix}.tag-${this._nodeKey++}`,
          control,
        ),
      );
    }

    pushLiteralText(text: string) {
      const last = this.result[this.result.length - 1];
      if (typeof last === 'string') {
        this.result[this.result.length - 1] += text;
      } else {
        this.result.push(text);
      }
    }

    pushObject(value: React.ReactNode) {
      this.result.push(value);
    }

    finish(): React.ReactNode[] {
      return this.result;
    }
  };
}

/**
 * A type representing any message that has been formatted using one of the
 * default React formatting methods in the `@discord/intl` system. Use this
 * type when you don't care about whether a message is static or contains rich
 * text or has dynamically-formatted values, when you need to handle both in
 * tandem, or when you need a generic return type for a function returning "any
 * already-formatted React message".
 *
 * Generally, this is like `React.ReactNode`, accepting both strings and rich
 * `ReactElement`s, but it is more specific to disallow nullish values and
 * other special React elements like `ReactPortal`. Seeing this type, you can
 * be confident that the value is intended to be the result of formatting an
 * intl message, even if the actual value comes from elsewhere (like a
 * user-generated string).
 */
export type ReactIntlMessage =
  // This is _not_ a branded string for the sake of compatibility in end-user code. Because these
  // are plain strings, there's still no risk of users providing arbitrary nodes here (unless they
  // forcibly or mistakenly downcast to `strings` themselves.
  // In the future, this could become `ReactIntlPlainString` instead.
  | string
  // This is explicitly `ReactElement | string` and _not_ `ReactNode`, because nullish values and
  // all other types it accepts (boolean, number, etc.) are _not_ valid children of a message.
  | ReactIntlRichText;

/**
 * A branded type representing a plain string that has been rendered by the React formatter. This
 * type should generally _not_ be used as a type constraint unless _absolute certainty_ that a
 * message was formatted is desirable. Instead, accept plain `string`, and this will be compatible.
 */
export type ReactIntlPlainString = string & { __brand: 'discord-intl' };

/**
 * While `ReactIntlMessage` represents the result of rendering _any_ message with the React
 * formatter, `ReactIntlRichText` specifically represents a message that was rendered and contains
 * rich text, meaning the result contains React nodes itself and represents a CST of the message.
 */
export type ReactIntlRichText = Array<React.ReactElement | string> & {
  __brand: 'discord-intl';
};

export function formatReact(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
  Builder: FormatBuilderConstructor<React.ReactElement>,
): ReactIntlMessage {
  if (typeof message === 'string') return message as ReactIntlPlainString;

  const parts = this.bindFormatValues(Builder, message, values);
  return parts as ReactIntlRichText;
}

/**
 * Create a new React formatter with the given rich text elements replacing the defaults. Use this
 * function to inject custom components for things like links and paragraphs, which may be best
 * suited to use components from a Design System or other library rather than native DOM elements.
 */
export function makeReactFormatter(
  richTextElements: RichTextFormattingMap<ReactFunctionTypes['hook']>,
): FormatterImplementation<ReactFunctionTypes, ReactIntlMessage, React.ReactNode> {
  return {
    format: formatReact,
    builder: createReactBuilder(richTextElements),
  };
}

export const reactFormatter = makeReactFormatter(DEFAULT_REACT_RICH_TEXT_ELEMENTS);
