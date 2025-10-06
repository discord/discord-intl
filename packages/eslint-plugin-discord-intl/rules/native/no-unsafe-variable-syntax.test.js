// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const noUnsafeVariableSyntax = require('./no-unsafe-variable-syntax');

const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  parserOptions: { ecmaVersion: 2015, sourceType: 'module' },
});

/**
 * @param {string} messages
 */
function defineMessages(messages) {
  return `
    import {defineMessages} from '@discord/intl';

    export default defineMessages(${messages});
  `;
}

ruleTester.run('no-unsafe-variable-syntax', noUnsafeVariableSyntax, {
  valid: [
    {
      name: 'normal strings',
      code: defineMessages("{ A: 'no trimmed whitespace' }"),
    },
  ],
  invalid: [
    {
      name: 'unsafe placeholder',
      code: defineMessages('{ A: `!!{foo}!!` }'),
      errors: 1,
    },
    {
      name: 'markdown content',
      code: defineMessages('{ A: `**!!{username}!!**` }'),
      errors: 1,
    },
    {
      name: 'number',
      code: defineMessages('{ A: `!!{count, number}!!` }'),
      errors: 1,
    },
    {
      name: 'date',
      code: defineMessages('{ A: `!!{tomorrow, date}!!` }'),
      errors: 1,
    },
    {
      name: 'unsafe plural',
      code: defineMessages('{ A: `!!{foo, plural, one {bar} other {baz}}!!` }'),
      errors: 1,
    },
    {
      name: 'nested within plural',
      code: defineMessages('{ A: `{foo, plural, one {bar !!{count}!!} other {baz}}` }'),
      errors: 1,
    },
    {
      name: 'double nested within plural',
      code: defineMessages('{ A: `!!{foo, plural, one {bar !!{count}!!} other {baz}}!!` }'),
      errors: 2,
    },
  ],
});

console.log('All tests passed!');
