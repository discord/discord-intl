const { isMessageDefinitionsFile, hashMessageKey } = require('@discord/intl-message-database');

/**
 * Babel plugin for obfuscating and minifying intl message _usages_, like
 * turning `intl.format(messages.SOME_LONG_MESSAGE_KEY_NAME)` into
 * `intl.format(messages['f21c/3'])`. This transform gets applied to every
 * file in the project, but only member expressions of imports from
 * `.messages.js` files are considered and affected.
 *
 * @param {any} babel - The Babel core object.
 * @returns {{visitor: import("babel__traverse").Visitor}} A visitor object for the Babel transform.
 */
module.exports = function metroIntlTransformerPlugin(babel) {
  /** @type {{types: import("@babel/types")}} */
  const { types: t } = babel;

  return {
    visitor: {
      ImportDeclaration(path, _state) {
        const importSource = path.node.source.value;
        // This transformer only handles usages of intl messages, so only
        // imports of definitions files need to be handled.
        if (!isMessageDefinitionsFile(importSource)) {
          return;
        }

        const defaultImport = path.node.specifiers.find(
          (specifier) => specifier.type === 'ImportDefaultSpecifier',
        );
        const bindingName = defaultImport?.local.name;
        if (bindingName == null) {
          return;
        }

        const binding = path.scope.bindings[bindingName];
        for (const reference of binding.referencePaths) {
          const parent = reference.parent;
          // We only care about member expressions that use the `.` syntax or
          // use string literals, since other syntaxes could be invalid.
          if (
            parent.type !== 'MemberExpression' ||
            (parent.computed && parent.property.type !== 'StringLiteral')
          ) {
            continue;
          }

          // We just want the actual value of the member being accessed.
          /** @type {string} */
          const memberName = (() => {
            switch (parent.property.type) {
              case 'StringLiteral':
                return parent.property.value;
              case 'Identifier':
                return parent.property.name;
              default:
                throw new Error(
                  '[INTL] Encountered a member expression with neither an identifier nor string literal member node',
                );
            }
          })();

          // Then hash it up and re-write the member with the hashed version.
          parent.computed = true;
          parent.property = t.stringLiteral(hashMessageKey(memberName));
        }
      },
    },
  };
};
