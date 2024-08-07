import type { InternalIntlMessage } from './message';
import type { IntlMessageGetterAdditions } from './message-loader';

/**
 * Use this function to define messages as part of the `@discord/intl`
 * system, with type checking support.
 *
 * This function does not actually exist, and gets compiled out as part
 * of the bundling process both on web and mobile. See the
 * `@discord/intl/README.md` for more information.
 */
export declare function defineMessages(messages: object): object;

/**
 * A map of tag names to render functions to apply when the tags are
 * encountered while formatting a message. Each `IntlManager` can be
 * constructed with a different map of these elements, for example to allow
 * one manager to render React output while another renders accessibility text.
 */
export type RichTextElementMap = Record<string, (chunks: any[]) => any>;

/**
 * Messages with no rich formatting and no value interpolations are just plain
 * strings, and are able to skip multiple formatting steps to work more
 * efficiently. This type represents those messages distinctly.
 */
type PlainIntlMessage = string;
/**
 * A richly-formatted message that injects dynamic values while formatting,
 * with the types of those values defined by the `FormatValues` object. All
 * messages of this type must have at least one value in the object, otherwise
 * the message should be typed as a `PlainIntlMessage`.
 */
type TypedIntlMessage<FormatValues extends object> = InternalIntlMessage & {
  values: FormatValues;
};

type AnyIntlMessage<FormatValues extends object | undefined = undefined> =
  FormatValues extends object ? TypedIntlMessage<FormatValues> : PlainIntlMessage;

/**
 * Getter function that retrieves a message that best matches the requested
 * locale. `FormatValues` represents either an object containing all of the
 * value types that are required to format the message, or `undefined` to
 * represent that there are no values required and the message is a plain
 * string.
 */
export type TypedIntlMessageGetter<FormatValues extends object | undefined> =
  IntlMessageGetterAdditions & {
    (locale: string): AnyIntlMessage<FormatValues>;
  };

/**
 * Common utility type for converting a union type to an intersection of all
 * the types in that union.
 */
type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (k: infer I) => void
  ? I
  : never;

/**
 * Given a message getter of type `T` (which may be a union of multiple message
 * types), return a combined type representing the intersection of all values
 * needed to satisfy the mix of values.
 *
 * For example, given a variable representing one of two message getters, where
 * one requires a single value to be formatted, but the other requires an
 * additional value on top of that one:
 *
 * ```typescript
 * // message1: "Hello {username}"
 * // message2: "Hello {username}, it's {currentDate, date}"
 * const message = someCondition ? message1 || message2;
 * ```
 *
 * Attempting to format this message would require that both `username` _and_
 * `currentDate` are supplied, so that all possible cases are covered, which is
 * what this type resolves:
 *
 * ```typescript
 * type ResolvedValuesTypes = FormatValuesFor<typeof message>;
 * // => {username: string, currentDate: Date}
 * ```
 *
 * This values type can then be used to enforce that all values are passed to
 * functions that format the message, like `i18n.format`:
 *
 * ```typescript
 * declare function format<T>(message: T, values: FormatValuesFor<T>): string;
 * ```
 *
 * ## Mixed message types
 *
 * If T is a union of message types including both rich messages requiring
 * values _and_ plain string messages with no values, this type will still
 * properly resolve all of the required values, assuming that the getters for
 * the plain messages use the convention of passing `undefined` as the generic
 * argument, representing "no values should be passed for this message".
 *
 *
 * ## Why this type?
 *
 * All "messages" are really defined as functions that return a specific
 * value of the message for a requested locale, `(locale: string) => Message`.
 * This is named `TypedIntlMessageGetter<T>`, where `T` is the definition of
 * the values required to format the message.
 *
 * This type requires being passed the entire getter function rather than the
 * returned message types, as narrowing to the returned type prevents
 * TypeScript from distributing the union properly. This surfaces as error
 * messages showing up on the _message_ type rather than the _value_ type when
 * they don't match, which doesn't give the user any helpful information about
 * what's wrong. This method ensures the value type is always the one being
 * checked against the provided message types, and error messages will be
 * accurate as a result.
 */
type FormatValuesFor<T> =
  // `[]` syntax forces TypeScript to not distribute the possible union of
  // types, meaning this represents "_all_ values of T are an extension of
  // the parent type", which in this case represents messages with no
  // format values. If that condition is matched, then `FormatValues` _cannot_
  // be supplied, represented by the `never`. This prevents accidentally
  // passing an object, even an empty one, for those strings, and allows the
  // formatter to optimize a little more on each call.
  [T] extends [TypedIntlMessageGetter<undefined>]
    ? never
    : // The `never` condition needs to be repeated here, but this time _without_
      // the `[]` syntax, so that only the union elements with actual values are
      // included in the union. Without this, the `infer U` on the latter side
      // fails to resolve and the type just becomes `never`.
      UnionToIntersection<
        T extends TypedIntlMessageGetter<undefined>
          ? never
          : T extends TypedIntlMessageGetter<infer U>
            ? U
            : never
      >;
