const NativeRules = require('./rules/native-rules');
module.exports = {
  rules: {
    'no-avoidable-exact-plurals': NativeRules.NoAvoidableExactPlurals,
    'no-duplicate-message-keys': NativeRules.NoDuplicateMessageKeys,
    'no-invalid-plural-selector': NativeRules.NoInvalidPluralSelector,
    'no-non-exhaustive-plurals': NativeRules.NoNonExhaustivePlurals,
    'no-missing-plural-other': NativeRules.NoMissingPluralOther,
    'no-repeated-plural-names': NativeRules.NoRepeatedPluralNames,
    'no-repeated-plural-options': NativeRules.NoRepeatedPluralOptions,
    'no-trimmable-whitespace': NativeRules.NoTrimmableWhitespace,
    'no-unicode-variable-names': NativeRules.NoUnicodeVariableNames,
    'no-unnecessary-plural': NativeRules.NoUnnecessaryPlural,
    'no-unsafe-variable-syntax': NativeRules.NoUnsafeVariableSyntax,

    'use-static-access': require('./rules/use-static-access'),
    'no-opaque-messages-objects': require('./rules/no-opaque-messages-objects'),
  },
  configs: {
    recommended: {
      plugins: ['@discord/discord-intl'],
      rules: {
        // Native rules
        '@discord/discord-intl/no-avoidable-exact-plurals': 'error',
        '@discord/discord-intl/no-duplicate-message-keys': 'error',
        '@discord/discord-intl/no-invalid-plural-selector': 'error',
        '@discord/discord-intl/no-non-exhaustive-plurals': 'error',
        '@discord/discord-intl/no-missing-plural-other': 'error',
        '@discord/discord-intl/no-repeated-plural-names': 'error',
        '@discord/discord-intl/no-repeated-plural-options': 'error',
        '@discord/discord-intl/no-trimmable-whitespace': 'error',
        '@discord/discord-intl/no-unicode-variable-names': 'error',
        '@discord/discord-intl/no-unnecessary-plural': 'error',
        '@discord/discord-intl/no-unsafe-variable-syntax': 'error',

        // JS rules
        '@discord/discord-intl/use-static-access': 'error',
        '@discord/discord-intl/no-opaque-messages-objects': 'error',
      },
    },
  },
};
