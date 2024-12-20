# metro-intl-transformer

A Metro loader for intl message definition files using `@discord/intl`. This loader handles both definitions and translations as a single group, emitting the appropriate file types and contents based on the kind of file provided.

Note that you'll also want/need the `@discord/babel-plugin-transform-discord-intl` plugin applied to your JS compilation to ensure that message usages compile to the same keys that match the compiled definitions.
