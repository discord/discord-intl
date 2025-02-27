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

import {
  parseDateTimeSkeleton,
  parseNumberSkeleton,
  parseNumberSkeletonFromString,
} from '@formatjs/icu-skeleton-parser';

import { AstNode, AstNodeIndices, FormatJsNodeType } from '@discord/intl-ast';
import type { RichTextTagNames } from './types';
import { FormatConfigType } from './data-formatters/config';
import { DataFormatters } from './data-formatters';

/**
 * Returns true if the tag name should be considered a rich text tag that
 * applies formatting to a message, rather than being a user-supplied value.
 */
function isRichTextTag(name: string) {
  return name[0] === '$';
}

export abstract class FormatBuilder<Result, ObjectType = Result extends object ? Result : never> {
  abstract pushRichTextTag(tag: RichTextTagNames, children: Result[], control: Result[]): void;
  abstract pushLiteralText(text: string): void;
  abstract pushObject(value: ObjectType): void;
  abstract finish(): Result[];
}

class MissingValueError extends Error {
  constructor(
    public variableName: string,
    public originalMessage: string,
    public nodeType: FormatJsNodeType,
  ) {
    super(
      `No value for variable '${variableName}' was provided for the localized message '${originalMessage}'`,
    );
  }
}

export type FormatBuilderConstructor<Result> = new () => FormatBuilder<Result>;

export function bindFormatValuesWithBuilder<
  T,
  ObjectType,
  Builder extends FormatBuilder<T, ObjectType>,
  FormatConfig extends FormatConfigType,
>(
  builder: Builder,
  nodes: AstNode[],
  locales: string | string[],
  formatters: DataFormatters<FormatConfig>,
  formatConfig: FormatConfig,
  values: Record<string, string | object> = {},
  currentPluralValue?: number,
  originalMessage?: string,
) {
  // Hot path for static messages that are just parsed as a single string element.
  if (nodes.length === 1 && typeof nodes[0] === 'string') {
    builder.pushLiteralText(nodes[0]);
    return;
  }

  let keyIndex = 0;
  for (const node of nodes) {
    if (typeof node === 'string') {
      builder.pushLiteralText(node);
      continue;
    }
    const nodeType = node[AstNodeIndices.Type];
    switch (nodeType) {
      case FormatJsNodeType.Pound:
        // Replace `#` in plural rules with the actual numeric value. Only
        // numeric values are replaced, otherwise the value is completed ignored?
        // Behavior copied from FormatJS directly.
        if (typeof currentPluralValue === 'number') {
          const value = formatters.formatNumber(currentPluralValue);
          builder.pushLiteralText(value);
        }
        continue;
    }

    const variableName = node[AstNodeIndices.Value];
    // Enforce that all required values are provided by the caller, even if the
    // actual value is falsy/undefined.
    if (!(variableName in values) && !isRichTextTag(variableName)) {
      throw new MissingValueError(variableName, originalMessage, nodeType);
    }

    const value = values[variableName];
    switch (nodeType) {
      case FormatJsNodeType.Argument:
        // `function` here captures ReactNodes, which can be functions that
        // return elements. Basically everything else fits in the `object`
        // type, and the generic type requires that an explicit ObjectType
        // be specified for cases where it's not. That means there's an
        // escape hatch that _might_ break this, but it would have to be
        // intentionally set by internal code to function that way, so we'll
        // operate on a promise that it won't for now.
        if (typeof value === 'object' || typeof value === 'function') {
          builder.pushObject(value as ObjectType);
        } else {
          // Taken from FormatJS: non-objects (strings, numbers, and falsy
          // values) all get cast to strings immediately as literal nodes.
          builder.pushLiteralText(String(value));
        }
        break;
      case FormatJsNodeType.Date: {
        const nodeStyle = node[AstNodeIndices.Style];
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          nodeStyle in formatConfig.date
            ? formatConfig.date[nodeStyle]
            : nodeStyle != null
              ? parseDateTimeSkeleton(nodeStyle)
              : undefined;
        builder.pushLiteralText(formatters.formatDate(value as number | Date, style));
        break;
      }
      case FormatJsNodeType.Time: {
        const nodeStyle = node[AstNodeIndices.Style];
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          nodeStyle in formatConfig.time
            ? formatConfig.time[nodeStyle]
            : nodeStyle != null
              ? parseDateTimeSkeleton(nodeStyle)
              : undefined;
        builder.pushLiteralText(formatters.formatTime(value as number | Date, style));
        break;
      }
      case FormatJsNodeType.Number: {
        const nodeStyle = node[AstNodeIndices.Style];
        // Distinct from FormatJS: We don't currently parse the skeleton ahead of time in the AST,
        // so this manages parsing the skeleton as well before passing it onto the date formatter.
        const style =
          nodeStyle in formatConfig.number
            ? formatConfig.number[nodeStyle]
            : nodeStyle != null
              ? (parseNumberSkeleton(
                  parseNumberSkeletonFromString(nodeStyle),
                ) as Intl.NumberFormatOptions)
              : undefined;
        const scaledValue =
          // @ts-expect-error This is a weird cast that's not accurate, but works in the short term.
          typeof value !== 'number' ? (value as number) : (value as number) * (style?.scale ?? 1);
        builder.pushLiteralText(formatters.formatNumber(scaledValue, style));
        break;
      }

      case FormatJsNodeType.Tag: {
        const children = node[AstNodeIndices.Children];
        const control = node[AstNodeIndices.Control];
        const appliedChildren = bindFormatValues(
          builder.constructor as FormatBuilderConstructor<T>,
          children,
          locales,
          formatters,
          formatConfig,
          values,
          currentPluralValue,
        );
        const appliedControl =
          control != null
            ? bindFormatValues(
                builder.constructor as FormatBuilderConstructor<T>,
                control,
                locales,
                formatters,
                formatConfig,
                values,
                currentPluralValue,
              )
            : [];
        if (isRichTextTag(variableName)) {
          builder.pushRichTextTag(
            variableName as RichTextTagNames,
            appliedChildren,
            appliedControl,
          );
        } else {
          if (typeof value !== 'function') {
            throw `expected a function type for a Tag formatting value, ${variableName}. got ${typeof value}: ${value}`;
          }
          let chunks = value(appliedChildren, `${keyIndex++}`);
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
        const requestedOption = value as string;
        const options = node[AstNodeIndices.Options];
        const option = requestedOption in options ? options[requestedOption] : options.other;
        if (option == null) {
          throw `${requestedOption} is not a known option for select value ${variableName}. Valid options are ${Object.keys(options).join(', ')}`;
        }
        bindFormatValuesWithBuilder(builder, option, locales, formatters, formatConfig, values);
        break;
      }

      case FormatJsNodeType.Plural: {
        const options = node[AstNodeIndices.Options];
        const offset = node[AstNodeIndices.Offset];
        const pluralType = node[AstNodeIndices.PluralType];
        const option = (() => {
          const exactSelector = `=${value}`;
          if (exactSelector in options) return options[exactSelector];

          const rule = formatters.getPluralRules({ type: pluralType }).select(
            // @ts-expect-error Assert this `as number` properly.
            (value as number) - (offset ?? 0),
          );
          return options[rule] ?? options.other;
        })();

        if (option == null) {
          throw `${value} is not a known option for plural value ${variableName}. Valid options are ${Object.keys(options).join(', ')}`;
        }

        bindFormatValuesWithBuilder(
          builder,
          option,
          locales,
          formatters,
          formatConfig,
          values,
          // @ts-expect-error assert this `as number` properly
          (value as number) - (offset ?? 0),
        );
        break;
      }
    }
  }
}

export function bindFormatValues<Result, FormatConfig extends FormatConfigType>(
  Builder: FormatBuilderConstructor<Result>,
  nodes: string | AstNode[],
  locales: string | string[],
  dataFormatters: DataFormatters<FormatConfig>,
  formatConfig: FormatConfig,
  values: Record<string, string | object> = {},
  currentPluralValue?: number,
): Result[] {
  const builder = new Builder();
  if (typeof nodes === 'string') {
    builder.pushLiteralText(nodes);
    return builder.finish();
  } else {
    bindFormatValuesWithBuilder(
      builder,
      nodes,
      locales,
      dataFormatters,
      formatConfig,
      values,
      currentPluralValue,
    );
    return builder.finish();
  }
}
