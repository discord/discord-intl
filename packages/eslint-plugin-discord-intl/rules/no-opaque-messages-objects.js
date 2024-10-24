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
      noSpread: 'Avoid spreading a messages object into another object',
      noReferenceHolding:
        'Avoid holding references to entire message objects. Use messages individually',
    },
  },
  create(context) {
    return traverseMessageObjectReferences(context, (reference) => {
      const parent = reference.parent;
      switch (parent.type) {
        case 'CallExpression':
          context.report({
            node: reference,
            messageId: 'noObjectArgument',
          });
          return;
        case 'SpreadElement':
          context.report({
            node: reference,
            messageId: 'noSpread',
          });
          return;
        case 'Property':
          context.report({
            node: reference,
            messageId: 'noReferenceHolding',
          });
          return;
        case 'UnaryExpression':
          if (parent.operator === 'typeof') {
            context.report({
              node: parent,
              messageId: 'noTypeof',
            });
            return;
          }
      }

      if (isTypeScript(context)) {
        // @ts-expect-error TSNodes
        if (parent.type === 'TSTypeQuery') {
          context.report({
            node: parent,
            messageId: 'noTypeof',
          });
          return;
        }
      }

      // Any other expression, if it's not a message access, is "opaque", so we want to report on
      // it with _some_ kind of generic info.
      if (
        parent.type !== 'MemberExpression' &&
        // @ts-expect-error TSNodes
        parent.type !== 'TSQualifiedName'
      ) {
        context.report({
          node: reference,
          messageId: 'noReferenceHolding',
        });
      }
    });
  },
});
