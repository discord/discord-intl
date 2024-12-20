# intl_markdown_visitor

Abstract Visitor trait definition for the AST of `intl_markdown`. Any other crate can use this crate to define a new `Visitor` and drive a walk of a message AST to perform any kind of transformation or other operation. Visitors are _not_ mutable, meaning any transformation of the tree _must_ generate a new tree rather than mutating the existing one.

This is a library crate that is only built as part of another crate.
