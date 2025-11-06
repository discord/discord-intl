// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const { NoTrimmableWhitespace } = require('./native-rules');

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

ruleTester.run('no-trimmable-whitespace', NoTrimmableWhitespace, {
  valid: [
    {
      name: 'normal strings',
      filename: 'en-US.messages.js',
      code: defineMessages("{ A: 'no trimmed whitespace' }"),
    },
    {
      name: 'templates',
      filename: 'en-US.messages.js',
      code: defineMessages(
        '{ A: `no trimmed whitespace`, QUASI: `${  space  }`, MULTILINE: `hi\n  yes` }',
      ),
    },
    {
      name: 'multi-line',
      filename: 'en-US.messages.js',
      code: defineMessages(
        `{ A: \`no trimmed
        whitespace\`}`,
      ),
    },
    {
      name: 'object',
      filename: 'en-US.messages.js',
      code: defineMessages(
        '{ A: { message: "no whitespace", description: "  does not matter here  " }}',
      ),
    },
  ],
  invalid: [
    {
      filename: 'en-US.messages.js',
      code: defineMessages(`{
        A: '  leading whitespace',
        B: 'trailing whitespace  ',
        C: '  surrounding whitespace  ',
      }`),
      errors: 4,
    },
    {
      filename: 'en-US.messages.js',
      code: defineMessages(`{
        TABS: '\tleading whitespace',
        B: 'trailing whitespace\t',
        C: '\tsurrounding whitespace\t',
      }`),
      errors: 4,
    },
    {
      filename: 'en-US.messages.js',
      code: defineMessages(`{
        NEWLINES: '\\nleading whitespace',
        B: 'trailing whitespace\\n',
        C: '\\nsurrounding whitespace\\n',
      }`),
      errors: 4,
    },
    {
      filename: 'en-US.messages.js',
      code: defineMessages(`{
        MIXED: '\\n  \\t leading\\n  \\twhitespace  \\t\\n',
      }`),
      errors: 2,
    },
    {
      filename: 'en-US.messages.js',
      code: defineMessages(`{
        ONLY_BROKEN: '\\n \\t leading\\n  \\twhitespace  \\t\\n',
        VALID: 'valid string',
      }`),
      errors: 2,
    },
    {
      name: 'object',
      filename: 'en-US.messages.js',
      code: defineMessages(
        '{ A: { message: " no whitespace", description: "  does not matter here  " }}',
      ),
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
