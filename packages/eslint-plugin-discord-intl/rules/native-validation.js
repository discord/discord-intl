const { traverseMessageDefinitions } = require('../lib/traverse');
const { processDefinitionsFile, database } = require('@discord/intl-loader-core');

/** @typedef {import('@discord/intl-loader-core/types').IntlDiagnostic} IntlDiagnostic */

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    docs: {
      description: 'Reveal diagnostics from native validation using @discord/intl',
      category: 'Best Practices',
    },
  },
  create(context) {
    let hasProcessed = false;
    /** @type {{[key: string]: IntlDiagnostic[]}} */
    const validations = {};
    return traverseMessageDefinitions(context, (definition, value, _definition, name) => {
      // So long as this is a definitions file, process it once and perform the native validations
      // to be queried afterward.
      if (!hasProcessed) {
        processDefinitionsFile(context.filename, context.sourceCode.text, {
          processTranslations: false,
        });
        for (const diagnostic of database.validateMessages()) {
          if (diagnostic.file === context.filename) {
            (validations[diagnostic.key] ??= []).push(diagnostic);
          }
        }
        hasProcessed = true;
      }

      if (name == null || validations[name] == null) return;

      for (const diagnostic of validations[name]) {
        context.report({
          node: value,
          message: diagnostic.description,
        });
      }
    });
  },
});
