module.exports = {
  rules: {
    'trimmed-whitespace': require('./rules/trimmed-whitespace'),
    'use-static-access': require('./rules/use-static-access'),
    'native-validation': require('./rules/native-validation'),
    'no-opaque-messages-objects': require('./rules/no-opaque-messages-objects'),
  },
  configs: {
    recommended: {
      plugins: ['@discord/discord-intl'],
      rules: {
        '@discord/discord-intl/trimmed-whitespace': 'error',
        '@discord/discord-intl/native-validation': 'off',
        '@discord/discord-intl/use-static-access': 'error',
        '@discord/discord-intl/no-opaque-messages-objects': 'error',
      },
    },
  },
};
