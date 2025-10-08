const { traverseAndReportMatchingNativeValidations } = require('../../lib/native-validation');

module.exports = /** @type {import('eslint').Rule.RuleModule} */ ({
  meta: {
    fixable: 'code',
    docs: {
      description:
        'Disallow `=0` and `=1` plural selectors unless necessary, and replace `0` and `1` values with `#` of them.',
      category: 'Best Practices',
    },
  },
  create(context) {
    return traverseAndReportMatchingNativeValidations(
      context,
      (diagnostic) => diagnostic.name === 'NoAvoidableExactPlurals',
    );
  },
});
