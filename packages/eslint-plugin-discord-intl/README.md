# eslint-plugin-discord-intl

An ESLint (v8) plugin for linting intl messages in the `@discord/intl` system. Currently this linter only checks best
practices for messages like whitespace and access notation in other source files, but will eventually be able to lint
the content of messages themselves using the database's `validateMessages` API.

## Install

```shell
pnpm add -D @discord/eslint-plugin-discord-intl
```

## Configure

```javascript
module.exports = {
  // Extend the recommended plugin configuration
  extends: ['plugin:@discord/eslint-plugin-discord-intl/recommended'],
  // Optionally add configuration for all rules
  settings: {
    '@discord/discord-intl': {
      extraImports: {
        '@app/intl': ['t'],
      },
    },
  },
};
```

## Message metadata

The recommended configuration enables `no-template-metadata` for definition files. Use plain
single- or double-quoted strings for descriptions, tags, and other message or file metadata;
template literals in metadata are silently ignored by the legacy extractor. Template literals
remain supported as message values.

```javascript
import { defineMessages } from '@discord/intl';

export const meta = { description: 'Feature messages', tags: ['feature'] };
export default defineMessages({
  GREETING: { message: `Hello {name}`, description: 'Greeting shown after signing in.' },
});
```

The rule uses public ESLint APIs and can also be enabled through oxlint's JS plugin support:

```json
{
  "jsPlugins": [
    { "name": "@discord/discord-intl", "specifier": "@discord/eslint-plugin-discord-intl" }
  ],
  "rules": { "@discord/discord-intl/no-template-metadata": "error" }
}
```
