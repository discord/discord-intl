// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const nativeValidation = require('./native-validation');

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

ruleTester.run('native-validation', nativeValidation, {
  valid: [
    {
      name: 'normal strings',
      filename: '/absolute/test.messages.js',
      code: defineMessages("{ A: 'no trimmed whitespace' }"),
    },
  ],
  invalid: [
    {
      name: 'repeated plural name',
      filename: '/absolute/test.messages.js',
      code: defineMessages(`{
        A: '{count, plural, one {{count} thing}}',
      }`),
      errors: 1,
    },
    {
      name: 'repeated plural option',
      filename: '/absolute/test.messages.js',
      code: defineMessages(`{
        A: '{count, plural, one {# one} other {# other 1} other {# other 2}}',
      }`),
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
