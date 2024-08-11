# @discord/intl-loader-core

This package acts as a core set of utilities and processes that loaders and transformers can rely on to implement
translation discovery, compilation, and management in a consistent way. It implements a core transformer to compile a
source file into a loader runtime for `@discord/intl`, as well as functions for scanning the file system for
translations, watching for changes, emitting typescript type definitions, and more.

For the most part, consuming packages should never have to interact with the message database directly when using this
package, which allows changes in the native extension to be masked and swapped out as needed. However, a `database`
instance is exposed for cases where additional functionality is needed.
