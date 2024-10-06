const nativePath = require('node:path');
const { isMessageDefinitionsFile } = require('@discord/intl-loader-core');

/**
 * Babel visitor for traversing all intl message accesses in a file. This visitor can be applied to
 * every file in a project, but only member expressions of imports from `.messages.js` files and any
 * imports as specified in the `extraImports` configuration are considered and affected.
 *
 * Configuration for `extraImports` differs from swc-intl-message-transformer in that
 * paths here are resolved, _absolute_ paths to the desired file, since babel
 * will often have re-written the AST to resolve import aliases by the time
 * this plugin runs.
 *
 * @param {(node: import('@babel/types').MemberExpression, messageName: string | undefined) => void} callback
 * Callback invoked for every message access that's encountered.
 * @returns {import("babel__traverse").Visitor} A visitor object for a Babel transform.
 */
function traverseMessageAccesses(callback) {
  return {
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
        /** @type {string | undefined} */
        const messageName = (() => {
          switch (parent.property.type) {
            case 'StringLiteral':
              return parent.property.value;
            case 'Identifier':
              return parent.property.name;
            default:
              return undefined;
          }
        })();

        callback(parent, messageName);
      }
    },
  };
}

module.exports = { traverseMessageAccesses };
