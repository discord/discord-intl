const { traverseMessageDefinitions } = require('../lib/traverse');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    docs: {
      description:
        'Disallow using whole messages objects as singular values, through passing as arguments to functions, taking the type of the object, and more.',
      category: 'Best Practices',
    },
    messages: {
      noDuplicateKeys: '{{name}} has already been defined earlier in this file',
    },
  },
  create(context) {
    const foundNames = new Set();
    return traverseMessageDefinitions(context, (property, value, _definition, name) => {
      if (name == null) return;

      if (foundNames.has(name)) {
        context.report({ node: property, messageId: 'noDuplicateKeys', data: { name } });
      }

      foundNames.add(name);
    });
  },
});
