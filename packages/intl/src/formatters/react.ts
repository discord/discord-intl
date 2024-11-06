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
  RichTextTagNames,
} from '../types';
import * as React from 'react';
import { IntlManager } from '../intl-manager';
import { FormatBuilder, FormatBuilderConstructor } from '../format';

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

/**
 * Creates a new `FormatBuilder` class that constructs a React element tree
 * from the message using the given `richTextElements` to apply formatting.
 * This allows consumers to inject their own design system components and
 * overrides for rendering elements like links, paragraphs, and code blocks.
 * @param richTextElements
 */
function createReactBuilder(richTextElements: RichTextFormattingMap<ReactFunctionTypes['hook']>): {
  new (): FormatBuilder<React.ReactNode>;
} {
  return class extends FormatBuilder<React.ReactNode> {
    _nodeKey: number = 0;
    result: React.ReactNode[] = [];

    pushRichTextTag(
      tag: RichTextTagNames,
      children: React.ReactNode[],
      control: React.ReactNode[],
    ) {
      this.result.push(richTextElements[tag](children, `${this._nodeKey++}`, control));
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

// This is explicitly `ReactElement | string` and _not_ `ReactNode`, because nullish values and any
// other type that ReactNode accepts (boolean, number, etc.) are _not_ valid children of a message.
export type ReactIntlMessage = Array<React.ReactElement | string> & { __brand: 'discord-intl' };

export function formatReact(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
  Builder: FormatBuilderConstructor<React.ReactElement>,
): ReactIntlMessage {
  if (typeof message === 'string') {
    return [message] as ReactIntlMessage;
  }

  const parts = this.bindFormatValues(Builder, message, values);
  return parts as ReactIntlMessage;
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
