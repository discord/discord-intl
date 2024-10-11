/**
 * Returns true if the current execution is using a TypeScript parser, using a best-effort guess
 * based on the given context.
 * @param {import('eslint').Rule.RuleContext} context
 * @returns {boolean}
 */
function isTypeScript(context) {
  return context.parserPath?.includes('@typescript-eslint') ?? false;
}

module.exports = { isTypeScript };
