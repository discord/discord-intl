# swc-intl-message-transformer

An SWC plugin for transforming intl message _usages_ to obfuscated, minified strings, saving on bundle size and allowing for complete anonymity of string names. 

The runtime of this package is a single `.wasm` file, which is loaded as the main file when requiring the package.

# Development

Ensure you have run `rustup target add wasm32-unknown-unknown` to be able to build this project for the wasm target.

When you've made a change to the rust code and want to test it in a bundler, build it with `pnpm build`, this will invoke cargo and build the `swc_intl_message_transformer.wasm` file that is used as the plugin in SWC.

This plugin is only used in production builds, and is added as part of the `options.jsc.experimental.plugins` configuration wherever `builtin:swc-loader` is used. When the wasm file is rebuilt, you'll need to restart rspack for it to pick up the change, but then the next build should use the new version.
