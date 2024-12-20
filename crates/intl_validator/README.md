# intl_validator

A message Visitor that runs various validations against the message AST to ensure consistency, accuracy, and validity of the content. Results are exposed as a list of `Diagnostic`s that can be serialized and interpreted by clients, like an adapter for ESLint and others.

This is a library crate that is only built as part of another crate.
