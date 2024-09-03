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
 * Placeholder type representing that this property is a Hook to be rendered
 * while formatting.
 */
export type HookFunction = { __brand: 'hook-function' };
/**
 * Placeholder type representing that this property is a Link to be rendered
 * while formatting.
 */
export type LinkFunction = { __brand: 'link-function' };
/**
 * Placeholder type representing that this property is a Handler function,
 * which will be attached to some event when the message is formatted.
 */
export type HandlerFunction = { __brand: 'handler-function' };

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
  FormatValues extends undefined ? PlainIntlMessage : TypedIntlMessage<FormatValues>;

/**
 * Getter function that retrieves a message that best matches the requested
 * locale. `FormatValues` represents either an object containing all of the
 * value types that are required to format the message, or `undefined` to
 * represent that there are no values required and the message is a plain
 * string.
 */
export interface TypedIntlMessageGetter<FormatValues extends object | undefined>
  extends IntlMessageGetterAdditions {
  (locale: string): AnyIntlMessage<FormatValues>;
}

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
 * needed to satisfy the mix of values defined in any of the messages.
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

/**
 * A template type for replaceable placeholder types to be defined by formatter
 * implementations. Messages are typed using placeholder types like
 * `HookFunction` and `HandlerFunction`, which are empty branded types
 * indicating the _intent_ of a property, which the formatter is then able to
 * replace with a more appropriate type. For example, a React formatter might
 * replace a `HookFunction` with `(content: ReactNode) => ReactNode` to allow
 * the caller to inject React elements within a string, while an AST formatter
 * might require a specific object shape, or allow any plain object.
 */
interface FunctionTypeMap {
  link: any;
  hook: any;
  handler: any;
}

type ReplaceKeepingNullability<T, P> = P | Exclude<T, NonNullable<T>>;

/**
 * Type implementation for applying a `FunctionTypeMap` to a given argument
 * type `T`. For each property of `T`, if its value is one of the placeholder
 * types, like `HookFunction`, it will be replaced by the corresponding
 * template value from `FunctionTypes`.
 */
type MapFunctionTypes<T, FunctionTypes extends FunctionTypeMap> = {
  [K in keyof T]: ReplaceKeepingNullability<
    T[K],
    NonNullable<T[K]> extends LinkFunction
      ? FunctionTypes['link']
      : NonNullable<T[K]> extends HookFunction
        ? FunctionTypes['hook']
        : NonNullable<T[K]> extends HandlerFunction
          ? FunctionTypes['handler']
          : T[K]
  >;
};

// Detect if T is explicitly `any`, in which case we don't want to do
// any replacements, since each type should be `any`.
// https://stackoverflow.com/a/61625831
type CheckStrictAny<Check, Yes, No> = (Check extends never ? true : false) extends false ? No : Yes;

/**
 * Given a `TypedIntlMessageGetter` type, return the format values type
 * argument from it. For example:
 *
 * ```typescript
 * type SomeMessage = TypedIntlMessageGetter<{name: string}>
 * type Result = IntlMessageGetterInnerType<SomeMessage>
 * // Result is `{name: string}`
 * ```
 */
type IntlMessageGetterInnerType<T extends TypedIntlMessageGetter<any>> =
  T extends TypedIntlMessageGetter<infer U> ? U : never;

/**
 * Create the format values argument type for a format function using the
 * configured `DefaultElements` for a formatter and injecting the placeholder
 * types from `FunctionTypes`. The result is the set of required values
 * necessary to format the message of type `T` with the calling formatter.
 */
type RequiredFormatValues<
  T extends TypedIntlMessageGetter<any>,
  DefaultElements,
  FunctionTypes extends FunctionTypeMap,
> = CheckStrictAny<
  IntlMessageGetterInnerType<T>,
  // If `T` is explicitly `any`, then individual properties can't be typed and
  // anything is allowed. This is only typical when consumers manually create a
  // `TypedIntlMessageGetter` type as needed to pass values around, but loses
  // any real type safety for the formatted values.
  // Without this, though, `MapFunctionTypes` would always match the first
  // value and turn every property into the mapped `LinkFunction` type, which
  // is almost always going to be wrong.
  any,
  MapFunctionTypes<Omit<FormatValuesFor<T>, keyof DefaultElements>, FunctionTypes>
>;
