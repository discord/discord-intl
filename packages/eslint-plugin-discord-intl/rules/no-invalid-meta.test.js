const { RuleTester } = require('eslint');
const noInvalidMeta = require('./no-invalid-meta');

const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  // Using 2022 to make sure the spread operator is supported.
  parserOptions: { ecmaVersion: 2022, sourceType: 'module' },
});

// @ts-ignore
const errorMessages = noInvalidMeta.meta.messages ?? {};

ruleTester.run('no-invalid-meta', noInvalidMeta, {
  valid: [
    {
      name: 'no meta defined',
      code: `
      import {defineMessages} from '@discord/intl';
      export default defineMessages({HELLO: "world"});
      `,
    },
    {
      name: 'not a messages file',
      code: `
      export const meta = {foobar: thisWouldBeInvalid};
      `,
    },
    {
      name: 'all literals',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = {
        project: 'some-feature',
        secret: false,
        translate: true,
        translationsPath: './translations',
      };

      export default defineMessages({HELLO: "world"});
      `,
    },
  ],
  invalid: [
    {
      name: 'no spread',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = {
        ...something,
      };

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.disallowSpread],
    },
    {
      name: 'no computed key',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = {
        [computed]: true,
      };

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.disallowComputedKey],
    },
    {
      name: 'no empty tags',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = {
        tags: [,'foo']
      };

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.invalidTagsValue],
    },
    {
      name: 'no variable tags',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = {
        tags: [aVariable]
      };

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.invalidTagsValue],
    },
    {
      name: 'no missing meta initializer',
      code: `
      import {defineMessages} from '@discord/intl';
      export let meta;

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.metaMustBeInitialized],
    },
    {
      name: 'require object literal initializer',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = false;

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.metaMustBeObjectLiteral],
    },
    {
      name: 'require object literal initializer',
      code: `
      import {defineMessages} from '@discord/intl';
      export const meta = someObject;

      export default defineMessages({HELLO: "world"});
      `,
      errors: [errorMessages.metaMustBeObjectLiteral],
    },
  ],
});

console.log('All tests passed!');
