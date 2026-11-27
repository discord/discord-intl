const { RuleTester } = require('eslint');
const noTemplateMetadata = require('./no-template-metadata');

const ruleTester = new RuleTester({
  parserOptions: { ecmaVersion: 2022, sourceType: 'module' },
});
const marker = "import {defineMessages} from '@discord/intl';\n";
const error = { messageId: 'ignoredTemplate', type: 'TemplateLiteral' };

ruleTester.run('no-template-metadata', noTemplateMetadata, {
  valid: [
    {
      name: 'plain metadata and template message values',
      code:
        marker +
        "export const meta = {description: 'File', tags: ['tag']};\n" +
        'export default defineMessages({' +
        "FIRST: {message: `Hello {name}`, description: 'Greeting', tags: ['tag']}," +
        "SECOND: {'message': `Hello`, 'description': 'Greeting'}," +
        'description: `This key is a message value`});',
    },
    {
      name: 'unrelated object is not metadata',
      code: marker + 'const unrelated = {description: `allowed`};',
    },
    {
      name: 'ordinary source file',
      filename: 'example.js',
      code: 'export const meta = {description: `allowed`};',
    },
  ],
  invalid: [
    {
      name: 'template descriptor description',
      code:
        marker +
        'export default defineMessages({HELLO: {message: `Hello`, description: `Greeting`}});',
      errors: [error],
    },
    {
      name: 'quoted property names',
      code:
        marker +
        "export default defineMessages({HELLO: {'message': `Hello`, 'description': `Greeting`}});",
      errors: [error],
    },
    {
      name: 'descriptor tags and other metadata',
      code:
        marker +
        'export default defineMessages({HELLO: {message: `Hello`, tags: [`tag`], secret: `false`, translate: `true`, other: `ignored`}});',
      errors: [error, error, error, error],
    },
    {
      name: 'file metadata values',
      code:
        marker +
        'export const meta = {description: `File`, tags: [`tag`], translationsPath: `./messages`, project: `ignored`};',
      errors: [error, error, error, error],
    },
    {
      name: 'interpolated metadata',
      code: marker + 'export const meta = {description: `File ${name}`};',
      errors: [error],
    },
    {
      name: 'entire meta initializer',
      code: marker + 'export const meta = `ignored`;',
      errors: [error],
    },
    ...['js', 'jsx', 'ts', 'tsx'].map((extension) => ({
      name: `definition filename .messages.${extension}`,
      filename: `Example.messages.${extension}`,
      code: 'export const meta = {description: `File`};',
      errors: [error],
    })),
  ],
});
