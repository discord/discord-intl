# swc-intl-message-transformer

An SWC plugin for transforming intl message _usages_ to obfuscated, minified strings, saving on bundle size and allowing for complete anonymity of string names.

The runtime of this package is a single `.wasm` file, which is loaded as the main file when requiring the package.

# Development

```
pnpm intl-cli swc build
```

This will automatically ensure you have the appropriate Rust toolchains and targets installed, compile the plugin to a `.wasm` file, and copy it to the appropriate location for use as an npm package.

# Usage

This plugin is usable in any project using SWC >= 1.0. Add it to the transpiler configuration through the `options.jsc.experimental.plugins` setting:

```js
{
  jsc: {
    experimental: {
      plugins: [
        [
          require.resolve('@discord/swc-intl-message-transformer'),
          // Optional extra configuration for customized usage.
          { extraImports: { './custom-module': ['additional', 'imported', 'names'] } },
        ],
      ];
    }
  }
}
```
