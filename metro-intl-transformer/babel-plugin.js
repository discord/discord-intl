const nativePath = require('node:path');
const { isMessageDefinitionsFile, hashMessageKey } = require('@discord/intl-loader-core');

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
    visitor: {
      ImportDeclaration(path, state) {
        const importSource = nativePath.resolve(
          // @ts-expect-error state is untyped but contains `file`
          nativePath.dirname(state.file.opts.filename),
          path.node.source.value,
        );
        const isDefinition = isMessageDefinitionsFile(importSource);
        // @ts-expect-error state is untyped but contains `opts` from the config.
        const extraImportSpecifiers = state.opts?.extraImports?.[importSource] ?? [];
        // This transformer only handles usages of intl messages, so only
        // imports of definitions files and configured extra specifiers need to
        // be handled.
        if (!isDefinition && extraImportSpecifiers.length === 0) {
          return;
        }

        const specifiers = path.node.specifiers
          .filter(
            (specifier) =>
              (isDefinition && specifier.type === 'ImportDefaultSpecifier') ||
              extraImportSpecifiers.includes(specifier.local.name),
          )
          .map((specifier) => specifier.local.name);

        const bindingReferences = specifiers.flatMap(
          (binding) => path.scope.bindings[binding]?.referencePaths ?? [],
        );
        for (const reference of bindingReferences) {
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
