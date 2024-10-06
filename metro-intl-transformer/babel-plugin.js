const { hashMessageKey } = require('@discord/intl-loader-core');
const { traverseMessageAccesses } = require('./babel-traverse-message-accesses');

/**
 * Babel plugin for obfuscating and minifying intl message _usages_, like
 * turning `intl.format(t.SOME_LONG_MESSAGE_KEY_NAME)` into `intl.format(t['f21c/3'])`.
 * This transform gets applied to every file in the project, but only member
 * expressions of imports from `.messages.js` files are considered and affected.
 *
 * Configuration for `extraImports` differs from swc-intl-message-transformer in that
 * paths here are resolved, _absolute_ paths to the desired file, since babel
 * will often have re-written the AST to resolve import aliases by the time
 * this plugin runs.
 *
 * @param {any} babel - The Babel core object.
 * @returns {{visitor: import("babel__traverse").Visitor}} A visitor object for the Babel transform.
 */
module.exports = function metroIntlTransformerPlugin(babel) {
  /** @type {{types: import("@babel/types")}} */
  const { types: t } = babel;

  return {
    visitor: traverseMessageAccesses((access, messageName) => {
      if (messageName == null) {
        throw new Error(
          '[INTL] Encountered a member expression with neither an identifier nor string literal member node',
        );
      }

      // Then hash it up and re-write the member with the hashed version.
      access.computed = true;
      access.property = t.stringLiteral(hashMessageKey(messageName));
    }),
  };
};
