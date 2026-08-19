const { isDefinitionsFile } = require('../lib/traverse');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    docs: {
      description:
        "Check that meta information is defined correctly and won't be silently ignored while processing.",
      category: 'Correctness',
    },
    messages: {
      metaMustBeInitialized: '`meta` must have an initializer',
      metaMustBeObjectLiteral: '`meta` must be defined using an object literal',
      disallowSpread: 'meta properties must be defined inline',
      disallowComputedKey: 'meta properties cannot be defined using computed keys',
      invalidTagsValue:
        '`tags` must be initialized using an array literal containing only string literals',
    },
  },
  create(context) {
    if (!isDefinitionsFile(context.filename, context.sourceCode.text)) return {};

    return {
      /**
       * @param {import('estree').VariableDeclarator} node
       */
      'VariableDeclarator[id.name="meta"]'(node) {
        if (node.init == null) {
          context.report({ node, messageId: 'metaMustBeInitialized' });
          return;
        }
        if (node.init.type !== 'ObjectExpression') {
          context.report({ node, messageId: 'metaMustBeObjectLiteral' });
          return;
        }

        const initializer = node.init;

        for (const property of initializer.properties) {
          if (property.type === 'SpreadElement') {
            context.report({ node: property, messageId: 'disallowSpread' });
            continue;
          }
          if (property.computed || property.key.type !== 'Identifier') {
            context.report({ node: property, messageId: 'disallowComputedKey' });
            continue;
          }
          switch (property.key.name) {
            // Tags are more than just a single literal. Every element of the array must also be a
            // string literal for it to be valid.
            case 'tags':
              if (property.value.type !== 'ArrayExpression') {
                context.report({ node: property.value, messageId: 'invalidTagsValue' });
                continue;
              }
              for (const tag of property.value.elements) {
                if (tag == null || tag.type !== 'Literal' || typeof tag.value !== 'string') {
                  context.report({ node: tag ?? property.value, messageId: 'invalidTagsValue' });
                }
              }
          }
        }
      },
    };
  },
});
