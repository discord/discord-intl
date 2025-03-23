const { traverseAndReportMatchingNativeValidations } = require('../../lib/native-validation');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    type: 'problem',
    docs: {
      description: 'Prevent message keys from being repeated across the entire database.',
    },
  },
  create(context) {
    return traverseAndReportMatchingNativeValidations(
      context,
      (diagnostic) => diagnostic.name === 'Processing::AlreadyDefined',
    );
  },
});
