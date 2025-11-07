const { traverseAndReportMatchingNativeValidations } = require('../lib/native-validation');

/** @import {Rule} from 'eslint' */
/**
 * @typedef NativeRuleDefinition
 * @property {string} diagnosticName
 * @property {Rule.RuleMetaData} meta
 */

/**
 * @param {NativeRuleDefinition} definition
 * @returns {Rule.RuleModule}
 */
function createNativeRule(definition) {
  return {
    meta: definition.meta,
    create(context) {
      return traverseAndReportMatchingNativeValidations(
        context,
        (diagnostic) => diagnostic.name === definition.diagnosticName,
      );
    },
  };
}

const NoAvoidableExactPlurals = createNativeRule({
  diagnosticName: 'NoAvoidableExactPlurals',
  meta: {
    fixable: 'code',
    docs: {
      description:
        'Disallow `=0` and `=1` plural selectors unless necessary, and replace `0` and `1` values with `#` of them.',
      category: 'Best Practices',
    },
  },
});
const NoDuplicateMessageKeys = createNativeRule({
  diagnosticName: 'Processing::AlreadyDefined',
  meta: {
    fixable: 'code',
    type: 'problem',
    docs: {
      description: 'Prevent message keys from being repeated across the entire database.',
      category: 'Correctness',
    },
  },
});
const NoInvalidPluralSelector = createNativeRule({
  diagnosticName: 'NoInvalidPluralSelector',
  meta: {
    fixable: 'code',
    type: 'problem',
    docs: {
      description: 'Disallow plural selectors that are not valid in the locale of the message',
      category: 'Correctness',
    },
  },
});
const NoNonExhaustivePlurals = createNativeRule({
  diagnosticName: 'NoNonExhaustivePlurals',
  meta: {
    fixable: 'code',
    type: 'problem',
    docs: {
      description: 'Enforce using `select` rather than `plural` when only exact selectors are used',
      category: 'Correctness',
    },
  },
});
const NoMissingPluralOther = createNativeRule({
  diagnosticName: 'NoMissingPluralOther',
  meta: {
    hasSuggestions: false,
    docs: {
      description:
        'Require that `plural` ICU expressions have an `other` selector to capture all possible cases',
      category: 'Correctness',
    },
  },
});
const NoRepeatedPluralNames = createNativeRule({
  diagnosticName: 'NoRepeatedPluralNames',
  meta: {
    fixable: 'code',
    hasSuggestions: false,
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
  },
});
const NoRepeatedPluralOptions = createNativeRule({
  diagnosticName: 'NoRepeatedPluralOptions',
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
  },
});
const NoTrimmableWhitespace = createNativeRule({
  diagnosticName: 'NoTrimmableWhitespace',
  meta: {
    fixable: 'code',
    hasSuggestions: true,
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
  },
});
const NoUnicodeVariableNames = createNativeRule({
  diagnosticName: 'NoUnicodeVariableNames',
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow whitespace at the beginning and end of intl messages',
      category: 'Best Practices',
    },
  },
});
const NoUnsafeVariableSyntax = createNativeRule({
  diagnosticName: 'NoUnsafeVariableSyntax',
  meta: {
    fixable: 'code',
    docs: {
      description: 'Disallow the obsoleted "unsafe variable" syntax (`!!{}!!`)',
      category: 'Best Practices',
    },
  },
});

module.exports = {
  NoAvoidableExactPlurals,
  NoDuplicateMessageKeys,
  NoInvalidPluralSelector,
  NoNonExhaustivePlurals,
  NoMissingPluralOther,
  NoRepeatedPluralNames,
  NoRepeatedPluralOptions,
  NoTrimmableWhitespace,
  NoUnicodeVariableNames,
  NoUnsafeVariableSyntax,
  createNativeRule,
};
