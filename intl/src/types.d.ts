import type { InternalIntlMessage } from './message';
import type { IntlMessageGetterAdditions } from './message-loader';
import { FormatBuilderConstructor } from './format';

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

export type IntlNumber = string | number | null | undefined;
export type IntlPlural = string | number | null | undefined;
export type IntlAny = any;
export type IntlDate = string | number | Date | null | undefined;
export type IntlTime = string | number | Date | null | undefined;

/**
 * Messages with no rich formatting and no value interpolations are just plain
 * strings, and are able to skip multiple formatting steps to work more
 * efficiently. This type represents those messages distinctly.
 */
export type PlainIntlMessage = string;
export type AnyIntlMessage = PlainIntlMessage | InternalIntlMessage;

export interface IntlMessageGetter extends IntlMessageGetterAdditions {
  (locale: string): InternalIntlMessage | PlainIntlMessage;
}

/**
 * Getter function that retrieves a message that best matches the requested
 * locale. `FormatValues` represents either an object containing all of the
 * value types that are required to format the message, or `undefined` to
 * represent that there are no values required and the message is a plain
 * string.
 */
export interface TypedIntlMessageGetter<FormatValues extends object | undefined>
  extends IntlMessageGetterAdditions {
  // TODO: This is lossy and unfortunate that typing can't be propagated
  // to the returned message type, but doing so causes problems with
  // contravariance of the type. When returning a message getter from a
  // function with a generic message getter return type, TypeScript will
  // try to assert that every returned type is strictly compatible with the
  // declared return type. In that case, if a message is missing any of the
  // properties declared in the type, it will error. But in the other direction
  // when passing a message getter as a parameter _into_ a function, TypeScript
  // will do the opposite and error if the getter has any _extra_ properties
  // from the declared type. The latter of these is definitely semantically
  // correct, but the former is more just an annoying part of the type system
  // because of this returned type being nested within this interface and the
  // strict subtyping that TypeScript uses.
  (locale: string): InternalIntlMessage;

  /**
   * A phantom type (not part of the actual typing of the object) that ensures
   * TypeScript won't use subtyping to widen the types of multiple messages
   * unless they are _identical_. For an example of the problem:
   * ```typescript
   * declare const message1: {a: string};
   * declare const message2: {a: string, b: boolean};
   *
   * const foo = condition ? message1 : message2;
   * // ^ This becomes just `{a: string}`, because it's the common base type.
   * ```
   *
   * This is fine for representing the object itself, but on messages those
   * types represent required values at runtime, and so the greatest outer
   * type is the only valid answer, rather than the greatest subtype.
   *
   * This phantom property ensures that TypeScript will never consider two
   * messages subtypes of each other unless they contain identical properties,
   * even the values of those properties, despite the fact that this only
   * considers names being passed in (not sure how, but it does). The result
   * is then always a union type of the possible types, which further
   * downstream will ensure that all format values are always checked for all
   * possible cases of the message.
   */
  __phantom: object extends FormatValues
    ? any
    : FormatValues extends undefined
      ? any
      : keyof FormatValues;
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
export type FormatValuesFor<T> =
  // `[]` syntax forces TypeScript to not distribute the possible union of
  // types, meaning this represents "_all_ values of T are an extension of
  // the parent type", which in this case represents messages with no
  // format values. If that condition is matched, then `FormatValues` _cannot_
  // be supplied, represented by the `never`. This prevents accidentally
  // passing an object, even an empty one, for those strings, and allows the
  // formatter to optimize a little more on each call.
  [T] extends [undefined]
    ? never
    : // The `never` condition needs to be repeated here, but this time _without_
      // the `[]` syntax, so that only the union elements with actual values are
      // included in the union. Without this, the `infer U` on the latter side
      // fails to resolve and the type just becomes `never`.
      UnionToIntersection<T extends undefined ? never : T>;

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
export interface FunctionTypeMap {
  link: any;
  hook: any;
  handler: any;
}

export interface FunctionTypes<Result, HandlerType = (content: Result[]) => Result | Result[]> {
  link: undefined | ((content: Result[]) => Result | Result[]);
  hook: undefined | ((content: Result[]) => Result | Result[]);
  handler: undefined | HandlerType;
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
    // TODO: replace these types with string literal types in the generated
    // file, turning this whole chain into `FunctionTypes[T[K]] ?? T[K]`.
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
export type IntlMessageGetterInnerType<T extends IntlMessageGetter> =
  T extends TypedIntlMessageGetter<infer U> ? U : never;

/**
 * Create the format values argument type for a format function using the
 * configured `DefaultElements` for a formatter and injecting the placeholder
 * types from `FunctionTypes`. The result is the set of required values
 * necessary to format the message of type `T` with the calling formatter.
 */
export type RequiredFormatValues<
  Values extends IntlMessageGetter,
  FunctionTypes extends FunctionTypeMap,
> =
  IntlMessageGetterInnerType<Values> extends undefined
    ? {}
    : CheckStrictAny<
        IntlMessageGetterInnerType<Values>,
        // If `T` is explicitly `any`, then individual properties can't be typed and
        // anything is allowed. This is only typical when consumers manually create a
        // `TypedIntlMessageGetter` type as needed to pass values around, but loses
        // any real type safety for the formatted values.
        // Without this, though, `MapFunctionTypes` would always match the first
        // value and turn every property into the mapped `LinkFunction` type, which
        // is almost always going to be wrong.
        any,
        MapFunctionTypes<FormatValuesFor<IntlMessageGetterInnerType<Values>>, FunctionTypes>
      >;

export interface RichTextFormattingMap<T = any> {
  $_: T;
  $b: T;
  $i: T;
  $p: T;
  $link: T;
  $code: T;
}

export type RichTextTagNames = keyof RichTextFormattingMap;

export interface FormatterImplementation<
  FunctionTypes extends FunctionTypeMap,
  Result,
  BuilderResult = Result,
> {
  format: (
    message: AnyIntlMessage,
    values: object,
    builder?: FormatBuilderConstructor<BuilderResult>,
  ) => Result;
  builder: FormatBuilderConstructor<BuilderResult extends (infer T)[] ? T : BuilderResult>;
  __$functionTypes?: FunctionTypes;
}
