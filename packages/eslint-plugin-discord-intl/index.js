module.exports = {
  rules: {
    'trimmed-whitespace': require('./rules/trimmable-whitespace'),
    'use-static-access': require('./rules/use-static-access'),
  },
  configs: {
    recommended: {
      plugins: ['@discord/discord-intl'],
      rules: {
        '@discord/discord-intl/trimmed-whitespace': 'error',
        '@discord/discord-intl/use-static-access': 'error',
      },
    },
  },
};
