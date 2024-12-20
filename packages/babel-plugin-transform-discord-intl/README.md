# babel-plugin-transform-discord-intl

A Babel plugin for transforming intl message _usages_ to obfuscated, minified strings, saving on bundle size and allowing for complete anonymity of string names.

# Development

This package is plain CommonJS with JSDoc types that requires no compilation. It can be tested locally by using `file:` or `link:` references from another project. No build step is required.

# Usage

This plugin is usable with Babel 7+. There is minimal configuration other than `extraImports` for adding additional names to check and transform usages for.

```js
[
  [
    require('@discord/babel-plugin-transform-discord-intl'),
    {
      extraImports: {
        [path.resolve('some/source/file')]: ['t', 'otherMessagesName'],
      },
    },
  ],
];
```
