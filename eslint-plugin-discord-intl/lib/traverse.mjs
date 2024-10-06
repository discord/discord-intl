import { isMessageDefinitionsFile } from '@discord/intl-loader-core';

/** @typedef {import('eslint').Rule.RuleListener} RuleListener */
/** @typedef {import('eslint').Rule.RuleContext} RuleContext */
/** @typedef {import('estree').MemberExpression} MemberExpression */
/** @typedef {import('estree').BaseModuleSpecifier} BaseModuleSpecifier */
/** @typedef {import('estree').ObjectExpression} ObjectExpression */
/** @typedef {import('estree').SimpleLiteral} SimpleLiteral */
/** @typedef {import('estree').TemplateLiteral} TemplateLiteral */
/** @typedef {import('estree').Property} Property */

/**
 * @typedef {(node: MemberExpression, importer: BaseModuleSpecifier) => void} MessageAccessCallback
 * @typedef {(
 *  property: Property,
 *  value: SimpleLiteral | TemplateLiteral,
 *  definition: ObjectExpression | undefined,
 * ) => void} MessageDefinitionCallback
 */

/**
 * Visit all `MemberExpression`s that act as accesses to intl messages, even if they are not
 * statically analyzable. All accesses on all imported message objects will be traversed in a
 * single pass, invoking `callback` for each one with the MemberExpression node as the first
 * argument, and the specifier that imported the messages object as the second.
 *
 * @param {RuleContext} context The rule context from ESLint
 * @param {MessageAccessCallback} callback Function to call for each instance of a message access
 * @returns {RuleListener} A visitor object for the Babel transform.
 */
function traverseMessageAccesses(context, callback) {
  const config = context.settings;
  const source = context.sourceCode;

  return /** @type {RuleListener} */ ({
    ImportDeclaration(path) {
      const importSource = /** @type {string} */ (path.source.value);
      // TODO: Make `isMessageDefinitionsFile` understand this properly.
      const isDefinition =
        isMessageDefinitionsFile(importSource) || importSource.endsWith('.messages');
      const extraImportSpecifiers = config.extraImports?.[importSource] ?? [];
      // This transformer only handles usages of intl messages, so only
      // imports of definitions files and configured extra specifiers need to
      // be handled.
      if (!isDefinition && extraImportSpecifiers.length === 0) {
        return;
      }

      const specifiers = path.specifiers.filter(
        (specifier) =>
          (isDefinition && specifier.type === 'ImportDefaultSpecifier') ||
          extraImportSpecifiers.includes(specifier.local.name),
      );

      for (const specifier of specifiers) {
        const bindingReferences = source
          .getDeclaredVariables(specifier)
          // This should only ever yield 1 variable, so flatMap is unnecessary, but it is safer to
          // use in case this changes in the future.
          .flatMap((variable) => variable.references);

        for (const reference of bindingReferences) {
          // @ts-expect-error `identifier` is actually `Identifier & NodeParentExtension`.
          const parent = reference.identifier.parent;

          // We only care about member expressions, since a direct reference to the
          // message source doesn't necessarily make it a message access.
          if (parent.type !== 'MemberExpression') continue;

          callback(/** @type {MemberExpression} */ parent, specifier);
        }
      }
    },
  });
}

/**
 * Visit all Message definitions in the file, invoking `callback` for each one with the string
 * value node as the first argument and, if present, the full definition object as the second
 * argument.
 *
 * @param {RuleContext} context The rule context from ESLint
 * @param {MessageDefinitionCallback} callback Function to call for each message definition.
 * @returns {RuleListener} A visitor object for the Babel transform.
 */
function traverseMessageDefinitions(context, callback) {
  const source = context.sourceCode;

  return /** @type {RuleListener} */ ({
    CallExpression(path) {
      // Only look at direct `defineMessages` function calls.
      if (path.callee.type !== 'Identifier' || path.callee.name !== 'defineMessages') return;

      // This is a very roundabout way of determining that the current CallExpression is calling
      // the `defineMessages` function that was directly imported from `@discord/intl`.
      const importingDefinitionSource = source
        .getScope(path)
        .variables[0].defs.find((definition) => definition.type === 'ImportBinding')?.parent
        .source.value;
      if (importingDefinitionSource !== '@discord/intl') return;

      const definitionsObject = path.arguments[0];
      // This early return assumes something else can/will assert that the call expression is valid.
      // In almost every case, that's TypeScript, so we don't need to report on it twice.
      if (definitionsObject.type !== 'ObjectExpression') return;

      for (const property of definitionsObject.properties) {
        // Spreads aren't analyzable here, and probably shouldn't be allowed.
        // TODO: Apply a lint when a Spread is encountered? Or follow it to the definition.
        if (property.type === 'SpreadElement') continue;

        switch (property.value.type) {
          case 'Literal':
          case 'TemplateLiteral':
            callback(
              property,
              /** @type {SimpleLiteral | TemplateLiteral} */ (property.value),
              undefined,
            );
            break;
          case 'ObjectExpression': {
            const messageProperty = property.value.properties.find(
              (prop) =>
                prop.type === 'Property' && 'value' in prop.key && prop.key.value === 'message',
            );
            if (
              messageProperty == null ||
              messageProperty.type === 'SpreadElement' ||
              messageProperty.value.type !== 'Literal'
            ) {
              break;
            }

            callback(
              property,
              /** @type {SimpleLiteral} */ (messageProperty.value),
              property.value,
            );
          }
        }
      }
    },
  });
}

export { traverseMessageAccesses, traverseMessageDefinitions };
