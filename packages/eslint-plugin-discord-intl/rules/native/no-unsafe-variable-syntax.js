const { traverseAndReportMatchingNativeValidations } = require('../../lib/native-validation');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow the obsoleted "unsafe variable" syntax (`!!{}!!`)',
      category: 'Best Practices',
    },
  },
  create(context) {
    return traverseAndReportMatchingNativeValidations(
      context,
      (diagnostic) => diagnostic.name === 'NoUnsafeVariableSyntax',
    );
  },
});
