const { traverseMessageAccesses } = require('../lib/traverse');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    docs: {
      description:
        'Ensure Intl Messages are not accessed with [] bracket notation, or that the Messages object is passed around, as these are not always statically analyzable',
    },
    messages: {
      computedAccess:
        'Messages should only be accessed with dot notation to be properly transformed during bundling',
    },
  },

  create(context) {
    return traverseMessageAccesses(context, (node) => {
      if (node.computed) {
        context.report({
          node,
          messageId: 'computedAccess',
          fix(fixer) {
            if (node.property.type === 'Literal' && typeof node.property.value === 'string') {
              const receiver = context.sourceCode.getText(node.object);
              const messageName = node.property.value;
              return fixer.replaceText(node, `${receiver}.${messageName}`);
            }
            return [];
          },
        });
      }
    });
  },
});
