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
