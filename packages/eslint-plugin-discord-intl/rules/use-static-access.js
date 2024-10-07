const { traverseMessageAccesses } = require('../lib/traverse');

/**
 * Output from import('identifier-regex').then((regex) => regex()). Used
 * directly because ESLint 8 doesn't support ESM plugins nicely, so we
 * have to stick in common JS.
 */
const IDENTIFIER_REGEX =
  /^(?!await|break|case|catch|class|const|continue|debugger|default|delete|do|else|enum|export|extends|false|finally|for|function|if|import|in|instanceof|new|null|return|super|switch|this|throw|true|try|typeof|var|void|while|with|yield|implements|interface|package|private|protected|public|arguments|eval|globalThis|Infinity|NaN|undefined)[$_\p{ID_Start}][$_\u200C\u200D\p{ID_Continue}]*$/u;

/**
 * Returns true if the given value is able to be written as an identifier rather than requiring a
 * string container.
 * @param {string} value
 * @returns {boolean}
 */
function isValidIdentifier(value) {
  return IDENTIFIER_REGEX.test(value);
}

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    docs: {
      description:
        'Ensure Intl Message accesses are statically analyzable by requiring dot notation, rather than computed accesses with `[]` brackets, using variables for message names, or interpolated template strings.',
      category: 'Best Practices',
    },
    messages: {
      computedAccess:
        'Messages should only be accessed with dot notation to be properly transformed during bundling',
      nonStringKey:
        'Message objects only contain names as keys. Any other type will always be incorrect',
      invalidName: 'Message names must be valid identifiers for use with dot access notation',
      interpolatedTemplate:
        'Template strings should not be used to access messages. Use a static name with dot access instead',
      staticTemplate:
        'Avoid template strings for message accesses to ensure they are always static',
    },
  },

  create(context) {
    return traverseMessageAccesses(context, (node) => {
      // If it's already using dot access, then every condition must already be met.
      if (!node.computed) return;

      const receiver = context.sourceCode.getText(node.object);
      const property = node.property;

      if (property.type === 'TemplateLiteral') {
        if (property.quasis.length === 1) {
          const messageName = property.quasis[0].value.raw;
          if (!isValidIdentifier(messageName)) {
            context.report({ node: property, messageId: 'invalidName' });
            return;
          }
          context.report({
            node,
            messageId: 'computedAccess',
            fix(fixer) {
              return fixer.replaceText(node, `${receiver}.${messageName}`);
            },
          });
        } else {
          context.report({
            node,
            messageId: 'interpolatedTemplate',
            // Not fixable because the interpolation isn't knowable at compile time.
          });
        }
        return;
      }

      if (property.type === 'Literal') {
        if (typeof property.value !== 'string') {
          context.report({ node: property, messageId: 'nonStringKey' });
          return;
        }
        const messageName = property.value;
        if (!isValidIdentifier(messageName)) {
          context.report({ node: property, messageId: 'invalidName' });
          return;
        }
        context.report({
          node,
          messageId: 'computedAccess',
          fix(fixer) {
            return fixer.replaceText(node, `${receiver}.${messageName}`);
          },
        });
        return;
      }

      context.report({
        node,
        messageId: 'nonStringKey',
      });
    });
  },
});
