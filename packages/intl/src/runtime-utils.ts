/**
 * Utility functions and classes used at runtime by the compiled output of bundlers using the intl
 * packages like `intl-loader-core`'s `MessageDefinitionsTransformer`.
 */
import { MessageLoader } from './message-loader';
import { IntlMessageGetter } from './types';

interface MessageBindsProxy {
  $$baseObject: Record<string, IntlMessageGetter>;
  $$loader: MessageLoader;
}

/** Type created by the message transformer when `bindMode` is set to `literal`. */
export type MessagesLiteral<T extends string> = Record<T, IntlMessageGetter>;
export type AnyIntlMessagesObject<T extends string> = MessageBindsProxy | MessagesLiteral<T>;

function isMessagesProxy(object: AnyIntlMessagesObject<any>): object is MessageBindsProxy {
  return object[Symbol.toStringTag] === 'IntlMessagesProxy';
}

/**
 * Return a new value that represents two message objects combined into one, regardless of their
 * kind (either proxies or object literals). The returned value will be able to access any message
 * contained in either of the two objects. This method generally assumes there is no overlap in
 * keys between the two message objects, and the order of resolution is undefined (varies based on
 * the types of the objects).
 *
 * Note that if both objects are proxies, the base loaders _are mutated in place_ to be chained
 * together. Any other usages of the first object will also automatically fall back to the second.
 */
export function chainMessagesObjects<
  const First extends MessagesLiteral<string>,
  const Second extends MessagesLiteral<string>,
>(first: First, second: Second): First & Second {
  const firstIsProxy = isMessagesProxy(first);
  const secondIsProxy = isMessagesProxy(second);

  // This explicit any is a little strange, but when the objects are proxies, they don't have any
  // actual type information, so trying to return them as a combination of `First` and `Second`
  // causes TypeScript to error saying there are possibly different instantiations. Casting to `any`
  // first makes it not care.
  let result: any = first;

  if (firstIsProxy && secondIsProxy) {
    // If both are proxies, no change is actually required, and the first proxy can just have its
    // loader set to fall back to the second.
    first.$$loader.fallbackWith(second.$$loader);
    result = first;
  } else if (!firstIsProxy && !secondIsProxy) {
    // If both objects are plain literals, they can just be spread together to get the result.
    result = { ...second, ...first };
  } else if (firstIsProxy && !secondIsProxy) {
    // If the first is a proxy and the second is an object, the second can be spread onto the first
    // to "pre-fill" values on it. In reality these cases should never be hit.
    result = Object.assign(first.$$baseObject, second);
  } else if (secondIsProxy && !firstIsProxy) {
    // And the same is true in reverse.
    result = Object.assign(second.$$baseObject, first);
  }

  return result as First & Second;
}

export function makeMessagesProxy(loader: MessageLoader): Record<string, IntlMessageGetter> {
  function makeBind(prop: string) {
    return (locale: string) => loader.get(prop, locale);
  }

  const baseObject = {};
  const proxy = new Proxy(baseObject, {
    ownKeys(self) {
      return Reflect.ownKeys(self);
    },
    getOwnPropertyDescriptor(self, prop) {
      self[prop] ||= makeBind(prop as string);
      return Reflect.getOwnPropertyDescriptor(self, prop);
    },
    get(self, prop) {
      if (prop === '$$typeof') {
        return 'object';
      }
      if (prop === Symbol.toStringTag) {
        return 'IntlMessagesProxy';
      }

      self[prop] ||= makeBind(prop as string);
      return self[prop];
    },
  });

  // Define the base object and loader on the proxy directly, but make them non-enumerable so that
  // they don't show up when using `Object.keys` or other accessors.
  Object.defineProperty(proxy, '$$baseObject', {
    value: baseObject,
    enumerable: false,
    configurable: false,
    writable: false,
  });
  Object.defineProperty(proxy, '$$loader', {
    value: loader,
    enumerable: false,
    configurable: false,
    writable: false,
  });
  return proxy;
}
