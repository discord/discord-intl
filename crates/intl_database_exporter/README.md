# intl_database_exporter

Database services for serializing message contents into various formats.

`export` is a generic service for persisting the entire _translation_ contents of a database to their appropriate files on the host system, according to the meta information for each message.

`bundle` is a specialized service for serializing a set of messages into a single file for a given locale, both for definitions and translations. These services are utilized by bundlers to compile source files into content that can be consistently included in bundled applications.

This is a library crate that is only built as part of another crate.
