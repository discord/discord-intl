// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const trimmableWhitespace = require('./trimmable-whitespace');

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

ruleTester.run('trimmable-whitespace', trimmableWhitespace, {
  valid: [
    {
      name: 'normal strings',
      code: defineMessages("{ A: 'no trimmable whitespace' }"),
    },
    {
      name: 'templates',
      code: defineMessages(
        '{ A: `no trimmable whitespace`, QUASI: `${  space  }`, MULTILINE: `hi\n  yes` }',
      ),
    },
    {
      name: 'multi-line',
      code: defineMessages(
        `{ A: \`no trimmable
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
      output: defineMessages(`{
        A: 'leading whitespace',
        B: 'trailing whitespace',
        C: 'surrounding whitespace',
      }`),
      errors: 3,
    },
    {
      code: defineMessages(`{
        TABS: '\tleading whitespace',
        B: 'trailing whitespace\t',
        C: '\tsurrounding whitespace\t',
      }`),
      output: defineMessages(`{
        TABS: 'leading whitespace',
        B: 'trailing whitespace',
        C: 'surrounding whitespace',
      }`),
      errors: 3,
    },
    {
      code: defineMessages(`{
        NEWLINES: '\\nleading whitespace',
        B: 'trailing whitespace\\n',
        C: '\\nsurrounding whitespace\\n',
      }`),
      output: defineMessages(`{
        NEWLINES: 'leading whitespace',
        B: 'trailing whitespace',
        C: 'surrounding whitespace',
      }`),
      errors: 3,
    },
    {
      code: defineMessages(`{
        MIXED: '\\n  \\t leading\\n  \\twhitespace  \\t\\n',
      }`),
      output: defineMessages(`{
        MIXED: 'leading\\n  \\twhitespace',
      }`),
      errors: 1,
    },
    {
      code: defineMessages(`{
        ONLY_BROKEN: '\\n \\t leading\\n  \\twhitespace  \\t\\n',
        VALID: 'valid string',
      }`),
      output: defineMessages(`{
        ONLY_BROKEN: 'leading\\n  \\twhitespace',
        VALID: 'valid string',
      }`),
      errors: 1,
    },
    {
      name: 'template quasis',
      code: defineMessages('{ A: `no trimmable whitespace`, QUASI: ` ${  space  } ` }'),
      output: defineMessages('{ A: `no trimmable whitespace`, QUASI: `${  space  }` }'),
      errors: 1,
    },
    {
      name: 'multiline templates',
      code: defineMessages('{ A: `no trimmable whitespace`, QUASI: `\n\t${  space  }\n  ` }'),
      output: defineMessages('{ A: `no trimmable whitespace`, QUASI: `${  space  }` }'),
      errors: 1,
    },
    {
      name: 'object',
      code: defineMessages(
        '{ A: { message: " no whitespace", description: "  does not matter here  " }}',
      ),
      output: defineMessages(
        '{ A: { message: "no whitespace", description: "  does not matter here  " }}',
      ),
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
