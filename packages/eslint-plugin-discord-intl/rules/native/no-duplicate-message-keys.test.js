const { RuleTester } = require('eslint');
const noDuplicateMessageKeys = require('./no-duplicate-message-keys');

const typescriptParser = require.resolve('@typescript-eslint/parser');
const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  parserOptions: { ecmaVersion: 2015, sourceType: 'module' },
});

ruleTester.run('no-duplicate-message-keys', noDuplicateMessageKeys, {
  valid: [
    {
      name: 'different messages',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_TWO: 'world',
      });
      `,
    },
    {
      name: 'different message keys with same value',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_TWO: 'hello',
      });
      `,
    },
  ],
  invalid: [
    {
      name: 'repeated message key',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_ONE: 'world',
      });
      `,
      errors: 1,
    },
    {
      name: 'repeated with others present',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_TWO: 'world',
        MESSAGE_ONE: 'world',
      });
      `,
      errors: 1,
    },
    {
      name: 'multiple repeated messages',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_TWO: 'world',
        MESSAGE_ONE: 'linting',
        MESSAGE_TWO: 'messages',
      });
      `,
      errors: 2,
    },
    {
      name: 'repeated with the same value',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({
        MESSAGE_ONE: 'hello',
        MESSAGE_ONE: 'hello',
      });
      `,
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
