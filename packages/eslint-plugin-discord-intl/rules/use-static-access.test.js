const { RuleTester } = require('eslint');
const useStaticAccess = require('./use-static-access');

const ruleTester = new RuleTester({
  // Must use at least ecmaVersion 2015 because
  // that's when `const` variables were introduced.
  parserOptions: { ecmaVersion: 2015, sourceType: 'module' },
});

ruleTester.run('use-static-access', useStaticAccess, {
  valid: [
    {
      name: 'non-messages expressions',
      code: `
        // All valid because they aren't message identifiers
        something['WHATEVER'];
        foo['bar'];
        t['correct'];
      `,
    },
    {
      name: 'dot-access message accesses',
      code: `
        import messages from 'Feature.messages';
        messages.SOME_MESSAGE;
        messages.SOME_MESSAGE['afterward'];
        parent[messages.SOME_MESSAGE];
        parent.call(messages.SOME_MESSAGE);
      `,
    },
    {
      name: 'extra imports',
      settings: {
        extraImports: {
          '@app/intl': ['t'],
        },
      },
      code: `
        import {t} from '@app/intl';
        t.SOME_MESSAGE;
        t.SOME_MESSAGE['afterward'];
        parent[t.SOME_MESSAGE];
        parent.call(t.SOME_MESSAGE);
      `,
    },
  ],
  invalid: [
    {
      name: 'computed literal access',
      code: `
        import messages from 'Feature.messages';
        messages['SOME_MESSAGE'];
      `,
      output: `
        import messages from 'Feature.messages';
        messages.SOME_MESSAGE;
      `,
      errors: 1,
    },
    {
      name: 'static template literal access',
      code: `
        import messages from 'Feature.messages';
        messages[\`SOME_MESSAGE\`];
      `,
      output: `
        import messages from 'Feature.messages';
        messages.SOME_MESSAGE;
      `,
      errors: 1,
    },
    {
      name: 'dynamic template literal access',
      code: `
        import messages from 'Feature.messages';
        messages[\`SCOPE_\${name}\`];
      `,
      errors: 1,
    },
    {
      name: 'variable access',
      code: `
        import messages from 'Feature.messages';
        messages[someVariable];
      `,
      errors: 1,
    },
    {
      name: 'invalid identifier access',
      code: `
        import messages from 'Feature.messages';
        messages['not an identifier'];
      `,
      errors: 1,
    },
    {
      name: 'keyword access',
      code: `
        import messages from 'Feature.messages';
        messages['return'];
      `,
      errors: 1,
    },
  ],
});

console.log('All tests passed!');
