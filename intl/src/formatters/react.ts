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
  $_: () => '',
  $b: (content) => h('strong', undefined, content),
  $code: (content) => h('code', undefined, content),
  $i: (content) => h('em', undefined, content),
  // $link will always be [href, <empty>, ...content]
  $link: ([href, ...content]) => h('a', { href }, content),
  $p: (content) => h('p', undefined, content),
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
    result: React.ReactNode[] = [];

    pushRichTextTag(tag: RichTextTagNames, children: React.ReactNode[]) {
      this.result.push(richTextElements[tag](children));
    }

    pushLiteralText(text: string) {
      this.result.push(text);
    }

    pushObject(value: object) {
      // @ts-expect-error this is technically invalid, but we'll just assume that if a format returns
      // an object, it'll be acting as some form of ReactNode.
      this.result.push(value);
    }

    finish(): React.ReactNode[] {
      return this.result;
    }
  };
}

export function formatReact(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
  Builder: FormatBuilderConstructor<React.ReactElement>,
): React.ReactElement {
  if (typeof message === 'string') {
    return React.createElement(React.Fragment, undefined, message);
  }

  const parts = this.bindFormatValues(Builder, message, values);
  return React.createElement(React.Fragment, undefined, parts);
}

/**
 * Create a new React formatter with the given rich text elements replacing the defaults. Use this
 * function to inject custom components for things like links and paragraphs, which may be best
 * suited to use components from a Design System or other library rather than native DOM elements.
 */
export function makeReactFormatter(
  richTextElements: RichTextFormattingMap<ReactFunctionTypes['hook']>,
): FormatterImplementation<ReactFunctionTypes, React.ReactElement, React.ReactNode> {
  return {
    format: formatReact,
    builder: createReactBuilder(richTextElements),
  };
}

export const reactFormatter = makeReactFormatter(DEFAULT_REACT_RICH_TEXT_ELEMENTS);
