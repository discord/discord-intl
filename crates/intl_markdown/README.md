# intl_markdown

Core Markdown parsing implementation for `discord-intl`. Generally speaking, this implementation follows the CommonMark spec as closely as possible, with additions and extensions added to accommodate integrating ICU syntax for dynamic messages. This combination parser produces a fully-static syntax tree that can be analyzed ahead of time for accurate type generation, segmentation, validation, and more.

The parser is implemented as a hand-built event-based parser. Source content is first analyzed as a whole for the overall block structure, which is then used to drive a loop of the inline parser. Parsing yields an Event buffer, which is then iterated to create a Concrete Syntax Tree of the content. Because Markdown output is non-linear with the input (e.g., spaces and newlines can change meaning or be omitted, escape characters are replaced, etc.), another conversion to an AST then happens that applies the necessary transformations to yield a final, static tree with the desired output. This AST is what all downstream libraries and services operate on.

## Development

There are many tests for this crate to ensure there are no regressions against the CommonMark spec or any of our extensions to the syntax. They are broken out into various cargo commands that can be run from this directory:

```shell
# Run all tests in this directory
cargo test-all
# Just run Markdown tests
cargo test-markdown
# Run an arbitrary subset of tests
cargo test-subset tests::qualified::module::name
```

The aliases for these tests are defined in `./cargo/config.toml`.

Additionally, a set of benchmarks is available to compare against `pulldown-cmark`, a standard CommonMark implementation in Rust known for being fast and efficient.

```shell
cargo bench
```
