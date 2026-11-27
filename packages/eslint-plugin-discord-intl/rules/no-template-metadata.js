const { isDefinitionsFile } = require('../lib/traverse');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    type: 'problem',
    docs: {
      description: 'Require plain quoted strings for message metadata.',
      category: 'Correctness',
    },
    schema: [],
    messages: {
      ignoredTemplate: 'Template literals are ignored in message metadata; use plain quotes.',
    },
  },
  create(context) {
    if (!isDefinitionsFile(context.filename, context.sourceCode.text)) return {};

    /** @param {import('estree').TemplateLiteral} node */
    function report(node) {
      context.report({ node, messageId: 'ignoredTemplate' });
    }

    return {
      'ExportNamedDeclaration > VariableDeclaration > VariableDeclarator[id.name="meta"] TemplateLiteral':
        report,
      'ExportDefaultDeclaration > CallExpression > ObjectExpression.arguments > Property > ObjectExpression.value > Property:not([key.name="message"]):not([key.value="message"]) TemplateLiteral':
        report,
    };
  },
});
