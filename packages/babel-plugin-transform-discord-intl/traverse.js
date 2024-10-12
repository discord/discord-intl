const nativePath = require('node:path');
const { isMessageDefinitionsFile } = require('@discord/intl-loader-core');

/** @typedef {import('babel__traverse').Scope} Scope */
/** @typedef {import('babel__traverse').NodePath} NodePath */

/**
 * Return all references to any of the given `names` that are accessible in `scope`.
 * @param {string[]} names
 * @param {Scope} scope
 * @returns {Array<NodePath>}
 */
function getBindingReferences(names, scope) {
  return names.flatMap((name) => scope.bindings[name]?.referencePaths ?? []);
}

/**
 * Returns the list of identifiers that can act as Messages objects if they are imported from
 * `source`, based on the configuration provided for this visitor.
 * @param {string} source
 * @param {unknown} state
 * @returns {[string[], boolean]} A tuple of the list of allowed specifiers and a boolean indicating whether the default import is allowed.
 */
function getAllowedMessageObjectIdentifers(source, state) {
  // @ts-expect-error state is untyped but contains `opts`
  const importSource = state.opts.preserveImportSource
    ? source
    : nativePath.resolve(
        // @ts-expect-error state is untyped but contains `file`
        nativePath.dirname(state.file.opts.filename),
        source,
      );
  const isDefinition = isMessageDefinitionsFile(importSource);
  // @ts-expect-error state is untyped but contains `opts` from the config.
  const extraImportSpecifiers = state.opts?.extraImports?.[importSource] ?? [];
  return [extraImportSpecifiers, isDefinition];
}

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
  /**
   * @param {NodePath[]} references
   */
  function traverseReferences(references) {
    for (const reference of references) {
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
  }

  return {
    ImportDeclaration(path, state) {
      const [extraImports, allowDefault] = getAllowedMessageObjectIdentifers(
        path.node.source.value,
        state,
      );
      const specifiers = path.node.specifiers
        .filter(
          (specifier) =>
            (allowDefault && specifier.type === 'ImportDefaultSpecifier') ||
            extraImports.includes(specifier.local.name),
        )
        .map((specifier) => specifier.local.name);

      const bindingReferences = getBindingReferences(specifiers, path.scope);
      traverseReferences(bindingReferences);
    },

    CallExpression(path, state) {
      const call = path.node;
      if (
        call.callee.type !== 'Identifier' ||
        call.callee.name !== 'require' ||
        call.arguments.length !== 1
      ) {
        return;
      }
      const modulePath = call.arguments[0];
      if (modulePath.type !== 'StringLiteral') return;
      if (typeof modulePath.value !== 'string') return;

      // TODO: Inspect bindings created by these requires and traverse them as well.
      // const [extraImports, allowDefault] = getAllowedMessageObjectIdentifers(
      //   modulePath.value,
      //   state,
      // );
    },
  };
}

module.exports = { traverseMessageAccesses };
