# @discord/jest-processor-discord-intl

A processor plugin for Jest to handle intl message definitions and translations from `@discord/intl`.

In your Jest config, add this as a transformer for message file patterns:

```javascript
module.exports = {
  transform: {
    // Order matters here! Even though this is an Object, Jest still processes these first to
    // last (object insertion order). These patterns can overlap, so we want to put all of the
    // special handling first.
    [INTL_MESSAGES_FILE_PATTERN]: require.resolve('@discord/jest-processor-discord-intl'),
    // ...other transforms, like `*.tsx?` and more.
  },
};
```

You'll also need to use the babel plugin to transform consuming code like normal bundling:

```javascript
// In a custom processor or wherever you configure Babel for Jest:
babel.transform({
  // ...
  plugins: [
    [
      require.resolve('@discord/babel-plugin-transform-discord-intl'),
      {
        // Jest does _not_ resolve these to absolute paths, so tell the plugin to use the
        // original import paths instead of resolving them like it does for Metro.
        preserveImportSource: true,
        extraImports: {
          '@app/intl': ['t', 'untranslated', 'international'],
        },
      },
    ],
  ],
});
```
