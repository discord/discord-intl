const { traverseAndReportMatchingNativeValidations } = require('../../lib/native-validation');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
  },
  create(context) {
    return traverseAndReportMatchingNativeValidations(
      context,
      (diagnostic) => diagnostic.name === 'NoRepeatedPluralNames',
    );
  },
});
