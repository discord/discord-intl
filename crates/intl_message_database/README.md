# intl_message_database

A Rust-based native Node extension module for quickly parsing, processing, and generating structured message definitions across an entire project. This is the core module powering all of the loaders, transformer plugins, and other tooling that relate to messages and translations. It includes a custom parser to handle a combination of ICU MessageFormat syntax for variable interpolations, along with (almost) full support for Markdown's inline styling syntax, and is able to output reformatted strings, ASTs, type definitions, perform validations across multiple translations, and more.

This crate is primarily developed for the Node API, but is implemented in a way that allows for other interfaces in the future (or splitting into multiple crates).

Business logic for this crate should be exposed as a Rust interface through the `public.rs` module. Interfaces for other languages should then be added as separate modules that proxy types from those languages to the `public` module. They should _not_ perform any other work or logic independently.

## Development

This crate is managed by the `intl-cli`, including commands for building and testing it across platforms:

```shell
# Build the library locally
pnpm intl-cli db build --target local
# Build specifically for a given target name
pnpm intl-cli db build --target darwin-arm64
# Run the benchmark tests on a local build to compare performance
pnpm intl-cli db bench
```
