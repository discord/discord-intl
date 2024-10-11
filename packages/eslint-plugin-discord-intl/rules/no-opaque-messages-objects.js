const { traverseMessageObjectReferences } = require('../lib/traverse');
const { isTypeScript } = require('../lib/is-typescript');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    docs: {
      description:
        'Disallow using whole messages objects as singular values, through passing as arguments to functions, taking the type of the object, and more.',
      category: 'Best Practices',
    },
    messages: {
      noObjectArgument:
        'Avoid passing message objects around as parameters. Use messages individually',
      noTypeof:
        'Avoid requesting the type of an entire messages object. Use messages individually.',
    },
  },
  create(context) {
    return traverseMessageObjectReferences(context, (reference) => {
      const parent = reference.parent;
      if (parent.type === 'CallExpression') {
        context.report({
          node: reference,
          messageId: 'noObjectArgument',
        });
        return;
      }

      if (parent.type === 'UnaryExpression' && parent.operator === 'typeof') {
        context.report({
          node: parent,
          messageId: 'noTypeof',
        });
        return;
      }

      if (isTypeScript(context)) {
        // @ts-expect-error TSNodes
        if (parent.type === 'TSTypeQuery') {
          context.report({
            node: parent,
            messageId: 'noTypeof',
          });
        }
      }
    });
  },
});
