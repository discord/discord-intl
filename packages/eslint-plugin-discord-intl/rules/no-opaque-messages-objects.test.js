// enforce-foo-bar.test.js
const { RuleTester } = require('eslint');
const noOpaqueMessagesObjects = require('./no-opaque-messages-objects');

const typescriptParser = require.resolve('@typescript-eslint/parser');
const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  parserOptions: { ecmaVersion: 2015, sourceType: 'module' },
});

ruleTester.run('no-opaque-messages-objects', noOpaqueMessagesObjects, {
  valid: [
    {
      name: 'normal message formatting',
      code: `
      import messages from 'Feature.messages';
      intl.format(messages.SOME_MESSAGE);
      otherFunction(messages.OTHER_MESSAGE);
      `,
    },
    {
      name: 'intermediate object passing',
      code: `
      import messages from 'Feature.messages';
      otherFunction({
        FIRST: messages.FIRST,
        OTHER_MESSAGE: messages.OTHER_MESSAGE,
      });
      `,
    },
    {
      name: 'typeof individual message',
      parser: typescriptParser,
      code: `
      import messages from 'Feature.messages';
      typeof messages.FOO;
      function foo(message: typeof messages.FOO) {}
      `,
    },
  ],
  invalid: [
    {
      name: 'passing whole object',
      code: `
      import messages from 'Feature.messages';
      otherFunction(messages);
      `,
      errors: 1,
    },
    {
      name: 'passing whole object',
      settings: {
        '@discord/discord-intl': {
          extraImports: { '@app/intl': ['t'] },
        },
      },
      code: `
      import {t} from '@app/intl';
      otherFunction(t);
      `,
      errors: 1,
    },
    {
      name: 'passing whole object',
      code: `
      import messages from 'Feature.messages';
      otherFunction(messages);
      `,
      errors: 1,
    },
    {
      name: 'typeof messages value',
      code: `
      import messages from 'Feature.messages';
      typeof messages;
      `,
      errors: 1,
    },
    {
      name: 'typeof messages as parameter',
      parser: typescriptParser,
      code: `
      import messages from 'Feature.messages';
      function foo(strings: typeof messages) {}
      `,
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
