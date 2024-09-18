/**
 * The core formatter, responsible for applying values to placeholders in
 * messages and creating a static CST from the result. This method does _not_
 * replace any of the rich-text formatting values (e.g., `$b` or `$link`) and
 * leaves those to final formatters to apply as appropriate for the context
 * of where the message is being rendered (e.g., React versus a plain string).
 *
 * The majority of this method is lifted from FormatJS' own `formatToParts`
 * implementation, but is abstracted here to provide a few benefits:
 * - working with the keyless AST format rather than having to transpose to the
 *   exact object format that FormatJS expects.
 * - understanding the difference between rich text formatting values and
 *   user-provided values.
 * - provide extra options related to the above for more control over how
 *   formatting operates, while abstracting out some of the common values.
 * - adding extra context for hook and link functions to provide a `key`
 *   parameter, important to have React treat the resulting elements nicely.
 */

import { FormatJsNode, FormatJsNodeType } from './keyless-json';
import { Formats, Formatters, MissingValueError } from 'intl-messageformat';
import { RichTextTagNames } from './types';
import {
  parseDateTimeSkeleton,
  parseNumberSkeleton,
  parseNumberSkeletonFromString,
} from '@formatjs/icu-skeleton-parser';

/**
 * Returns true if the tag name should be considered a rich text tag that
 * applies formatting to a message, rather than being a user-supplied value.
 */
function isRichTextTag(name: string) {
  return name[0] === '$';
}

export abstract class FormatBuilder<Result> {
  abstract pushRichTextTag(tag: RichTextTagNames, children: Result[]): void;
  abstract pushLiteralText(text: string): void;
  abstract pushObject(value: object): void;
  abstract finish(): Result[];
}

export type FormatBuilderConstructor<Result> = new () => FormatBuilder<Result>;

export function bindFormatValuesWithBuilder<T, Builder extends FormatBuilder<T>>(
  builder: Builder,
  nodes: FormatJsNode[],
  locales: string | string[],
  formatters: Formatters,
  formats: Formats,
  values: Record<string, string | object> = {},
  currentPluralValue?: number,
  originalMessage?: string,
) {
  // Hot path for static messages that are just parsed as a single string element.
  if (nodes.length === 1 && nodes[0].type === FormatJsNodeType.Literal) {
    builder.pushLiteralText(nodes[0].value);
    return;
  }

  for (const node of nodes) {
    switch (node.type) {
      case FormatJsNodeType.Literal:
        builder.pushLiteralText(node.value);
        continue;

      case FormatJsNodeType.Pound:
        // Replace `#` in plural rules with the actual numeric value. Only
        // numeric values are replaced, otherwise the value is completed ignored?
        // Behavior copied from FormatJS directly.
        if (typeof currentPluralValue === 'number') {
          const value = formatters.getNumberFormat(locales).format(currentPluralValue);
          builder.pushLiteralText(value);
        }
        continue;
    }

    const { value: variableName } = node;
    // Enforce that all required values are provided by the caller, even if the
    // actual value is falsy/undefined.
    if (!(variableName in values) && !isRichTextTag(variableName)) {
      throw new MissingValueError(variableName, originalMessage);
    }

    const value = values[variableName];

    switch (node.type) {
      case FormatJsNodeType.Argument:
        // Empty values don't need to be added at all, they are purely for AST representation.
        if (variableName == '$_') break;

        if (typeof value === 'object') {
          builder.pushObject(value);
        } else {
          // Taken from FormatJS: non-objects (strings, numbers, and falsy
          // values) all get cast to strings immediately as literal nodes.
          builder.pushLiteralText(String(value));
        }
        break;
      case FormatJsNodeType.Date: {
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          node.style in formats.date
            ? formats.date[node.style]
            : node.style != null
              ? parseDateTimeSkeleton(node.style)
              : undefined;
        // @ts-expect-error Cast string values to dates properly.
        builder.pushLiteralText(formatters.getDateTimeFormat(locales, style).format(value));
        break;
      }
      case FormatJsNodeType.Time: {
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          node.style in formats.time
            ? formats.time[node.style]
            : node.style != null
              ? parseDateTimeSkeleton(node.style)
              : undefined; // TODO: parseSkeleton();
        builder.pushLiteralText(
          // @ts-expect-error Cast string values to dates properly.
          formatters.getDateTimeFormat(locales, style).format(value),
        );
        break;
      }
      case FormatJsNodeType.Number: {
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          node.style in formats.number
            ? formats.number[node.style]
            : node.style != null
              ? parseNumberSkeleton(parseNumberSkeletonFromString(node.style))
              : undefined;
        // @ts-expect-error Support `scale` style property.
        const scaledValue = style?.scale != 1 ? (value as number) * style.scale : (value as number);
        builder.pushLiteralText(formatters.getNumberFormat(locales, style).format(scaledValue));
        break;
      }

      case FormatJsNodeType.Tag: {
        const { children } = node;
        const appliedChildren = bindFormatValues(
          builder.constructor as FormatBuilderConstructor<T>,
          children,
          locales,
          formatters,
          formats,
          values,
          currentPluralValue,
        );
        if (isRichTextTag(variableName)) {
          builder.pushRichTextTag(variableName as RichTextTagNames, appliedChildren);
        } else {
          if (typeof value !== 'function') {
            throw `expected a function type for a Tag formatting value, ${variableName}. got ${typeof value}: ${value}`;
          }
          let chunks = value(appliedChildren);
          chunks = Array.isArray(chunks) ? chunks : [chunks];
          for (const chunk of chunks) {
            if (typeof chunk === 'string') {
              builder.pushLiteralText(chunk);
            } else {
              builder.pushObject(chunk);
            }
          }
        }
        break;
      }

      case FormatJsNodeType.Select: {
        const option = node.value in node.options ? node.options[node.value] : node.options.other;
        if (option == null) {
          throw `${node.value} is not a known option for select value ${variableName}. Valid options are ${Object.keys(node.options).join(', ')}`;
        }
        bindFormatValuesWithBuilder(builder, option.value, locales, formatters, formats, values);
        break;
      }

      case FormatJsNodeType.Plural: {
        const option = (() => {
          const exactSelector = `=${value}`;
          if (exactSelector in node.options) return node.options[exactSelector];
          const rule = formatters
            .getPluralRules(locales, { type: node.pluralType })
            // @ts-expect-error Assert this `as number` properly.
            .select((value as number) - (node.offset ?? 0));
          return node.options[rule] ?? node.options.other;
        })();

        if (option == null) {
          throw `${node.value} is not a known option for plural value ${variableName}. Valid options are ${Object.keys(node.options).join(', ')}`;
        }
        bindFormatValuesWithBuilder(
          builder,
          option.value,
          locales,
          formatters,
          formats,
          values,
          // @ts-expect-error assert this `as number` properly.
          (value as number) - (node.offset ?? 0),
        );
        break;
      }
    }
  }
}

export function bindFormatValues<Result>(
  Builder: FormatBuilderConstructor<Result>,
  nodes: FormatJsNode[],
  locales: string | string[],
  formatters: Formatters,
  formats: Formats,
  values: Record<string, string | object> = {},
  currentPluralValue?: number,
): Result[] {
  const builder = new Builder();
  bindFormatValuesWithBuilder(
    builder,
    nodes,
    locales,
    formatters,
    formats,
    values,
    currentPluralValue,
  );
  return builder.finish();
}
