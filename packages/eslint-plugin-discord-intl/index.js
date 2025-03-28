module.exports = {
  rules: {
    'no-repeated-plural-names': require('./rules/native/no-repeated-plural-names'),
    'no-repeated-plural-options': require('./rules/native/no-repeated-plural-options'),
    'no-trimmable-whitespace': require('./rules/native/no-trimmable-whitespace'),
    'no-unicode-variable-names': require('./rules/native/no-unicode-variable-names'),
    'no-duplicate-message-keys': require('./rules/native/no-duplicate-message-keys'),

    'use-static-access': require('./rules/use-static-access'),
    'no-opaque-messages-objects': require('./rules/no-opaque-messages-objects'),
  },
  configs: {
    recommended: {
      plugins: ['@discord/discord-intl'],
      rules: {
        // Native rules
        '@discord/discord-intl/no-trimmable-whitespace': 'error',
        '@discord/discord-intl/no-repeated-plural-names': 'error',
        '@discord/discord-intl/no-repeated-plural-options': 'error',
        '@discord/discord-intl/no-unicode-variable-names': 'error',

        // JS rules
        '@discord/discord-intl/use-static-access': 'error',
        '@discord/discord-intl/no-duplicate-message-keys': 'error',
        '@discord/discord-intl/no-opaque-messages-objects': 'error',
      },
    },
  },
};
