# discord-intl

Redefined internationalization support, created by Discord to manage millions of messages across multiple projects,
multiple programming languages, and multiple client platforms. `discord-intl` supports an expanded message format that combines ICU
MessageFormat syntax with Markdown to make authoring rich, stylized messages as clear and concise as possible.

This project was born out of a need for more flexibility with our translations, a desire for granular ownership, and a
need to refactor to keep up with modern versions of `intl-messageformat` and others that power the system.

This repository contains the full set of packages for managing translations, both at runtime and the tools to support
development:

- `@discord/intl` - The runtime and client side package that powers the actual usage of messages and translations in
  a project. This is also the package used for _defining_ messages that are able to be processed and handled by all of
  the other tools.
- `@discord/intl-message-database` - A standalone Rust/WASM package that parses a message definition file to extract
  the messages along with the resolved meta information for each message. This is what powers all of the development
  tooling, including the transformers.
- `@discord/swc-intl-message-transformer` - An SWC plugin written in Rust that transforms message _usages_ into
  production-ready versions throughout an entire project.
- `@discord/rspack-intl-loader` - Integration of the transformer for message _definitions_ in webpack/rspack, the
  bundler we use for webp projects. Responsible for finding translation files and doing the actual code transformation.
- `@discord/metro-intl-loader` - Integration of the transformer for message _definitions_ in Metro, the bundler we
  use for react-native projects.

# Usage

You most likely only need to know about how to use `defineMessages` to write new strings and string modules. A message
definition file _must_ have the extension `.messages.js`, and _must_ have a default export that calls `defineMessages`
inline, like this:

```typescript
// Import the defineMessages magic function from the package.
import { defineMessages } from '@discord/intl';

/**
 * Meta information about the strings contained in this file. Each string will
 * be tagged with this information by default, and it will be sent along to
 * translators to help provide context, categorization, and some special
 * features like hiding "secret" strings that shouldn't be bundled until a
 * specific release date.
 *
 * Future additions would be able to specify visual context for each string
 * (e.g. a screenshot image where the string is used) and more.
 */
export const meta = {
  project: 'custom-status',
  secret: true,
  translate: true,
};

/**
 * Messages are "defined" by creating a default export with `defineMessages`.
 * This function provides the typeguards for the shape of each message, and
 * ensures a consistent format for the external tooling to rely on when finding
 * all messages in the codebase.
 *
 * This _must_ be defined inline as a default export, and will be transformed
 * by bundlers both in development and production builds.
 */
export default defineMessages({
  HELLO_WORLD: {
    defaultMessage: 'Hello, world!',
    description: 'The standard greeting for new computers.',
  },
});
```

## Development

This assumes you have Rust installed and matching the `rust-toolchain.toml` in this repo.

This repository contains a CLI tool for easily managing all development and maintenance tasks. After cloning the repository, run:

```shell
# Install dependencies
pnpm i
# Run the cli
pnpm intl-cli --help
```

When working on the native code side in any of the Rust crates, tests most likely run through the `db` project:

```shell
# Build a local version of the database package
pnpm intl-cli db build --target local
# Benchmark the local database, building it fresh before running.
pnpm intl-cli db bench --build
```

Frontend packages sometimes need build steps as well, which can also be managed with the CLI:

```shell
# Build the intl runtime
pnpm intl-cli runtime build
# Build the SWC message transformer plugin
pnpm intl-cli swc build
```

All packages can be used locally in other projects as `file:` and `link:` dependencies. Note that the Metro bundler does not respect `link:` dependencies locally and will need to use `file:` and re-install the package every time a change is made.

## Releasing

The CLI also manages tasks for versioning and releasing packages for public use:

```shell
# Bump the version of all packages in the repo to prepare for release
pnpm intl-cli eco version bump --help
# Release a new canary build (only works for non-built projects)
pnpm intl-cli ci publish-canary @discord/intl [...and more packages]
# Publish a complete new release of all packages
pnpm intl-cli ci release
```
