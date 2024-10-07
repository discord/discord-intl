# intl-message-database

A Rust-based native Node extension module for quickly parsing, processing, and generating structured message definitions
across an entire project. This is the core module powering all of the loaders, transformer plugins, and other tooling
that relate to messages and translations. It includes a custom parser to handle a combination of ICU MessageFormat syntax
for variable interpolations, along with (almost) full support for Markdown's inline styling syntax, and is able to output
reformatted strings, ASTs, type definitions, perform validations across multiple translations, and more.