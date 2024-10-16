# @discord/rspack-intl-loader

A Webpack/Rspack loader for i18n message definition files using `@discord/intl`. This loader handles both definitions
and translations as a single group, emitting the appropriate file types and contents based on the kind of file provided.

## Usage

Add the loader as a rule for _all_ kinds of messages files in your application:

```javascript
const INTL_MESSAGES_REGEXP = /\.messages\.(js|json|jsona)$/;

rules = [
  {
    test: INTL_MESSAGES_REGEXP,
    loader: '@discord/rspack-intl-loader',
  },
];
```

Make sure that the intl loader is the _only_ transformation that happens on these files, or at least is the first one (
before e.g. minification). The loader relies on the structure of the raw source (with ES6 import and export statements,
for example) and will not work on transpiled modules. To do this, just be sure to `exclude` the file patterns from all
other JS transformations.

### Consumers

Note that you'll also want/need the `@discord/swc-intl-message-transformer` plugin applied to your JS compilation to
ensure that message usages compile to the same keys that match the compiled definitions.

```javascript
rules = [
  {
    loader: 'builtin:swc-loader',
    exclude: [INTL_MESSAGES_REGEXP],
    options: {
      jsc: {
        experimental: {
          // Just using a string here causes rspack/swc to error out immediately
          // with "failed to get node_modules path", seemingly because it doesn't
          // correctly give context for how the module should be resolved. Using
          // `require.resolve` directly creates an absolute path for rspack to load
          // directly instead.
          //
          // This plugin is responsible for obfuscating and minimizing intl message
          // _usages_ across every source file, to match the compiled _definitions_
          // that are handled by rspack-intl-loader in a separate rule.
          plugins: [
            [
              require.resolve('@discord/swc-intl-message-transformer'),
              // Add configuration for other message import sources as needed.
              { extraImports: { '@app/intl': ['t', 'untranslated', 'international'] } },
            ],
          ],
        },
      },
    },
  },
];
```

### Webpack compatibility

Webpack processes asset files slightly differently than Rspack and relies on some different conventions. Most notably,
Webpack's default transformations won't handle converting ESM back to CommonJS automatically (without configuring
another loader, for example). To handle this, use the `jsonExportMode` option for the loader to have it emit converted
CommonJS modules instead of ESM:

```javascript
rules = [
  {
    test: INTL_MESSAGES_REGEXP,
    use: [
      {
        loader: '@discord/rspack-intl-loader',
        // This should work to be compatible with Webpack builds
        options: { jsonExportMode: 'webpack' },
      },
    ],
  },
];
```
