import { traverseMessageDefinitions } from '../lib/traverse.mjs';

/**
 * @param {string} raw
 * @returns {string}
 */
function trimLeadingWhitespace(raw) {
  return raw.replace(/^(\s|\\[rtn])+/m, '');
}
/**
 * @param {string} raw
 * @returns {string}
 */
function trimTrailingWhitespace(raw) {
  return raw.replace(/(\s|\\[rtn])+$/m, '');
}

export default /** @type {import('eslint').Rule.RuleModule} */ ({
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
      const sourceText = context.sourceCode.getText(value);
      const quote = sourceText[0];
      const rawNoQuotes = sourceText.slice(1, -1);

      const trimmed = trimLeadingWhitespace(trimTrailingWhitespace(rawNoQuotes));
      if (trimmed === rawNoQuotes) return;

      context.report({
        node: value,
        messageId: 'noWhitespace',
        fix(fixer) {
          return [fixer.replaceText(value, `${quote}${trimmed}${quote}`)];
        },
      });
    });
  },
});
