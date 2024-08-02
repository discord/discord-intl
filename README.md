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
