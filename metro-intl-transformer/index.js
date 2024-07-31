const { isMessageDefinitionsFile, hashMessageKey } = require('@discordapp/intl-message-database');

/**
 * Babel plugin for barrel file export handling.
 *
 * @param {{types: import("@babel/types")}} babel - The Babel core object.
 * @returns {{visitor: import("babel__traverse").Visitor}} A visitor object for the Babel transform.
 */
module.exports = function metroIntlTransformerPlugin(babel) {
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
            !(
              parent.type === 'MemberExpression' ||
              (parent.computed && parent.property.type === 'StringLiteral')
            )
          ) {
            continue;
          }

          // We just want the actual value of the member being accessed.
          /** @type {string} */
          const memberName =
            parent.property.type === 'StringLiteral' ? parent.property.value : parent.property.name;

          // Then hash it up and re-write the member with the hashed version.
          parent.computed = true;
          parent.property = t.stringLiteral(hashMessageKey(memberName));
        }
      },
    },
  };
};
