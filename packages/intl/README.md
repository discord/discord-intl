# `@discord/intl`

Redefined internationalization support, created by Discord to manage millions of messages across multiple projects, multiple programming languages, and multiple client platforms. discord-intl supports an expanded message format that combines ICU MessageFormat syntax with Markdown to make authoring rich, stylized messages as clear and concise as possible.

This project was born out of a need for more flexibility with our translations, a desire for granular ownership, and a need to refactor to keep up with modern versions of `intl-messageformat` and others that power the system.

## Usage

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

This project is managed with the `intl-cli`:

```shell
# Compile the TypeScript source to distributable JavaScript once
pnpm intl-cli runtime build
# For development, run in watch mode to compile changes on the fly
pnpm intl-cli rt watch
```
