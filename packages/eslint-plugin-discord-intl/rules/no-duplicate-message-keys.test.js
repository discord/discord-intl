const { RuleTester } = require('eslint');
const { NoDuplicateMessageKeys } = require('./native-rules');

const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  parserOptions: { ecmaVersion: 2015, sourceType: 'module' },
});

ruleTester.run('no-duplicate-message-keys', NoDuplicateMessageKeys, {
  valid: [
    {
      name: 'different messages',
      filename: 'en-US.messages.js',
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
      filename: 'en-US.messages.js',
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
      filename: 'en-US.messages.js',
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
      filename: 'en-US.messages.js',
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
      filename: 'en-US.messages.js',
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
      filename: 'en-US.messages.js',
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
