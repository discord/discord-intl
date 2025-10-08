module.exports = {
  rules: {
    'no-avoidable-exact-plurals': require('./rules/native/no-avoidable-exact-plurals'),
    'no-repeated-plural-names': require('./rules/native/no-repeated-plural-names'),
    'no-repeated-plural-options': require('./rules/native/no-repeated-plural-options'),
    'no-trimmable-whitespace': require('./rules/native/no-trimmable-whitespace'),
    'no-unicode-variable-names': require('./rules/native/no-unicode-variable-names'),
    'no-duplicate-message-keys': require('./rules/native/no-duplicate-message-keys'),
    'no-unsafe-variable-syntax': require('./rules/native/no-unsafe-variable-syntax'),

    'use-static-access': require('./rules/use-static-access'),
    'no-opaque-messages-objects': require('./rules/no-opaque-messages-objects'),
  },
  configs: {
    recommended: {
      plugins: ['@discord/discord-intl'],
      rules: {
        // Native rules
        '@discord/discord-intl/no-avoidable-exact-plurals': 'error',
        '@discord/discord-intl/no-trimmable-whitespace': 'error',
        '@discord/discord-intl/no-repeated-plural-names': 'error',
        '@discord/discord-intl/no-repeated-plural-options': 'error',
        '@discord/discord-intl/no-unicode-variable-names': 'error',
        '@discord/discord-intl/no-unsafe-variable-syntax': 'error',

        // JS rules
        '@discord/discord-intl/use-static-access': 'error',
        '@discord/discord-intl/no-duplicate-message-keys': 'error',
        '@discord/discord-intl/no-opaque-messages-objects': 'error',
      },
    },
  },
};
