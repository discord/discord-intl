const { isMessageDefinitionsFile } = require('@discord/intl-message-database');

/** @typedef {import('eslint').Rule.NodeParentExtension} NodeParentExtension */
/** @typedef {import('eslint').SourceCode} SourceCode */
/** @typedef {import('eslint').Scope.ScopeManager} ScopeManager */
/** @typedef {import('eslint').Scope.Reference} Reference */
/** @typedef {import('eslint').Rule.RuleListener} RuleListener */
/** @typedef {import('eslint').Rule.RuleContext} RuleContext */
/** @typedef {import('estree').Node} Node */
/** @typedef {import('estree').MemberExpression} MemberExpression */
/** @typedef {import('estree').ImportDeclaration} ImportDeclaration */
/** @typedef {import('estree').ImportSpecifier} ImportSpecifier */
/** @typedef {import('estree').ImportDefaultSpecifier} ImportDefaultSpecifier */
/** @typedef {import('estree').ImportNamespaceSpecifier} ImportNamespaceSpecifier */
/** @typedef {import('estree').BaseModuleSpecifier} BaseModuleSpecifier */
/** @typedef {import('estree').ObjectExpression} ObjectExpression */
/** @typedef {import('estree').SimpleLiteral} SimpleLiteral */
/** @typedef {import('estree').TemplateLiteral} TemplateLiteral */
/** @typedef {import('estree').Property} Property */
/** @typedef {import('estree').Identifier} Identifier */

/**
 * @typedef DiscordIntlPluginConfig
 * @property {Record<string, string[]>} [extraImports]
 */

/**
 * @param {DiscordIntlPluginConfig | undefined} config
 * @param {ImportDeclaration} path
 * @returns {Array<ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier>}
 */
function getImportedMessagesObjectSpecifiers(config, path) {
  const importSource = /** @type {string} */ (path.source.value);
  // TODO: Make `isMessageDefinitionsFile` understand this properly.
  const isDefinition = isMessageDefinitionsFile(importSource) || importSource.endsWith('.messages');
  const extraImportSpecifiers = config?.extraImports?.[importSource] ?? [];
  // This transformer only handles usages of intl messages, so only
  // imports of definitions files and configured extra specifiers need to
  // be handled.
  if (!isDefinition && extraImportSpecifiers.length === 0) {
    return [];
  }

  return path.specifiers.filter(
    (specifier) =>
      (isDefinition && specifier.type === 'ImportDefaultSpecifier') ||
      extraImportSpecifiers.includes(specifier.local.name),
  );
}

/**
 *
 * @param {RuleContext} context
 * @param {Node} node
 * @returns {Reference[]}
 */
function getBindingReferences(context, node) {
  return (
    context.sourceCode
      .getDeclaredVariables(node)
      // This should only ever yield 1 variable, so flatMap is unnecessary, but it is safer to
      // use in case this changes in the future.
      .flatMap((variable) => /** @type {Reference[]} */ (variable.references))
  );
}

/**
 * @typedef {(
 *  node: MemberExpression & NodeParentExtension,
 *  importer: BaseModuleSpecifier & NodeParentExtension
 * ) => void} MessageAccessCallback
 * @typedef {(
 *  reference: Identifier & NodeParentExtension,
 *  importer: BaseModuleSpecifier & NodeParentExtension
 * ) => void} MessagesReferenceCallback
 * @typedef {(
 *  property: Property & NodeParentExtension,
 *  value: SimpleLiteral | TemplateLiteral,
 *  definition: ObjectExpression | undefined,
 *  name: string | undefined,
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
  const config = context.settings['@discord/discord-intl'];

  return /** @type {RuleListener} */ ({
    ImportDeclaration(path) {
      const specifiers = getImportedMessagesObjectSpecifiers(config, path);
      if (specifiers.length === 0) return;

      for (const specifier of specifiers) {
        const bindingReferences = getBindingReferences(context, specifier);

        for (const reference of bindingReferences) {
          const parent = /** @type {MemberExpression & NodeParentExtension} */ (
            // @ts-expect-error `identifier` is actually `Identifier & NodeParentExtension`.
            reference.identifier.parent
          );

          // We only care about member expressions, since a direct reference to the
          // message source doesn't necessarily make it a message access.
          if (parent.type !== 'MemberExpression') continue;

          callback(parent, /** @type {BaseModuleSpecifier & NodeParentExtension} */ (specifier));
        }
      }
    },
  });
}

/**
 * Returns true if the given file should be treated as a message definitions file, either by the
 * name of the file or by the content it includes.
 *
 * @param {string} fileName
 * @param {string} content
 */
function isDefinitionsFile(fileName, content) {
  if (isMessageDefinitionsFile(fileName)) return true;
  // Any file importing `defineMessages` _should_ be a message definitions file with
  // the expected structure.
  return content.match('import { ?defineMessages ?} from [\'"]@discord/intl[\'"]') != null;
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

        const name = /** @type {string | undefined } */ (
          (() => {
            const key = property.key;
            switch (key.type) {
              case 'Literal':
                return key.value;
              case 'Identifier':
                return key.name;
              default:
                return undefined;
            }
          })()
        );

        switch (property.value.type) {
          case 'Literal':
          case 'TemplateLiteral':
            callback(
              /** @type {Property & NodeParentExtension} */ (property),
              /** @type {SimpleLiteral | TemplateLiteral} */ (property.value),
              undefined,
              name,
            );
            break;
          case 'ObjectExpression': {
            const messageProperty = property.value.properties.find((prop) => {
              return prop.type === 'Property' && 'name' in prop.key && prop.key.name === 'message';
            });
            if (
              messageProperty == null ||
              messageProperty.type === 'SpreadElement' ||
              messageProperty.value.type !== 'Literal'
            ) {
              break;
            }

            callback(
              /** @type {Property & NodeParentExtension} */ (property),
              /** @type {SimpleLiteral} */ (messageProperty.value),
              property.value,
              name,
            );
          }
        }
      }
    },
  });
}

/**
 * Visit all `Identifier`s that act as accesses to a complete intl messages object. This includes
 * all references, including message accesses, object passing, and includes the original import
 * for completeness.
 *
 * @param {RuleContext} context The rule context from ESLint
 * @param {MessagesReferenceCallback} callback Function to call for each instance of a message access
 * @returns {RuleListener} A visitor object for the Babel transform.
 */
function traverseMessageObjectReferences(context, callback) {
  const config = context.settings['@discord/discord-intl'];

  return /** @type {RuleListener} */ ({
    ImportDeclaration(path) {
      const specifiers = getImportedMessagesObjectSpecifiers(config, path);
      for (const specifier of specifiers) {
        const bindingReferences = getBindingReferences(context, specifier);

        for (const reference of bindingReferences) {
          callback(
            /** @type {Identifier & NodeParentExtension} */ (reference.identifier),
            /** @type {BaseModuleSpecifier & NodeParentExtension} */ (specifier),
          );
        }
      }
    },
  });
}

module.exports = {
  isDefinitionsFile,
  traverseMessageAccesses,
  traverseMessageDefinitions,
  traverseMessageObjectReferences,
};
