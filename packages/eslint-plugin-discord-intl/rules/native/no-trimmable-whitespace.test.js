// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const noTrimmableWhitespace = require('./no-trimmable-whitespace');

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

ruleTester.run('no-trimmable-whitespace', noTrimmableWhitespace, {
  valid: [
    {
      name: 'normal strings',
      code: defineMessages("{ A: 'no trimmed whitespace' }"),
    },
    {
      name: 'templates',
      code: defineMessages(
        '{ A: `no trimmed whitespace`, QUASI: `${  space  }`, MULTILINE: `hi\n  yes` }',
      ),
    },
    {
      name: 'multi-line',
      code: defineMessages(
        `{ A: \`no trimmed
        whitespace\`}`,
      ),
    },
    {
      name: 'object',
      code: defineMessages(
        '{ A: { message: "no whitespace", description: "  does not matter here  " }}',
      ),
    },
  ],
  invalid: [
    {
      code: defineMessages(`{
        A: '  leading whitespace',
        B: 'trailing whitespace  ',
        C: '  surrounding whitespace  ',
      }`),
      errors: 4,
    },
    {
      code: defineMessages(`{
        TABS: '\tleading whitespace',
        B: 'trailing whitespace\t',
        C: '\tsurrounding whitespace\t',
      }`),
      errors: 4,
    },
    {
      code: defineMessages(`{
        NEWLINES: '\\nleading whitespace',
        B: 'trailing whitespace\\n',
        C: '\\nsurrounding whitespace\\n',
      }`),
      errors: 4,
    },
    {
      code: defineMessages(`{
        MIXED: '\\n  \\t leading\\n  \\twhitespace  \\t\\n',
      }`),
      errors: 2,
    },
    {
      code: defineMessages(`{
        ONLY_BROKEN: '\\n \\t leading\\n  \\twhitespace  \\t\\n',
        VALID: 'valid string',
      }`),
      errors: 2,
    },
    {
      name: 'object',
      code: defineMessages(
        '{ A: { message: " no whitespace", description: "  does not matter here  " }}',
      ),
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
