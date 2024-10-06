const { traverseMessageDefinitions } = require('../lib/traverse');
module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
    messages: {
      noWhitespace:
        'Do not add leading/trailing whitespace to messages. It can lead to inconsistency and ambiguity during translation',
    },
  },
  create(context) {
    return traverseMessageDefinitions(context, (_property, value, _definition) => {
      switch (value.type) {
        case 'TemplateLiteral': {
          const first = value.quasis[0].value.raw;
          const last = value.quasis[value.quasis.length - 1].value.raw;
          if (first.trimStart() === first && last.trimEnd() === last) {
            return;
          }

          context.report({
            node: value,
            messageId: 'noWhitespace',
          });
          return;
        }
        case 'Literal': {
          const raw = value.raw;
          if (typeof value.value !== 'string' || raw == null) return;
          if (value.value.trim() === value.value) return;

          context.report({
            node: value,
            messageId: 'noWhitespace',
            fix(fixer) {
              const quote = raw[0];
              const rawInner = raw.slice(1, raw.length - 1).trim();
              return [fixer.replaceText(value, `${quote}${rawInner}${quote}`)];
            },
          });
        }
      }
    });
  },
});
