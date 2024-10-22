/** @typedef {import('eslint').Rule.RuleListener} RuleListener */
/** @typedef {import('eslint').Rule.RuleContext} RuleContext */
/** @typedef {import("@discord/intl-loader-core/types").IntlDiagnostic} IntlDiagnostic */

const crypto = require('node:crypto');
const { processDefinitionsFile, database } = require('@discord/intl-loader-core');
const { traverseMessageDefinitions, isDefinitionsFile } = require('./traverse');
/** @type {Map<string, Record<string, IntlDiagnostic[]>>} */
const FILE_VALIDATIONS = new Map();

/**
 * Process the given file as a definitions file, run the native `validateMessages` function on the
 * database, and store all of the resulting validations.
 *
 * This function will only process a given file once. All future calls will return the cached
 * validations, allowing them to be reused across multiple rules without wasting work validating
 * the same messages multiple times.
 *
 * @param {string} fileName
 * @param {string} content
 * @return {Record<string, IntlDiagnostic[]>}
 */
function processAndValidateNative(fileName, content) {
  const fileKey = crypto.hash('sha1', content);
  const existing = FILE_VALIDATIONS.get(fileKey);
  if (existing != null) return existing;

  // ESLint will use a fake filename for dynamic input. in this case, we need to tell the
  // native database that this is a JS message definition _somehow_, so we'll make a random
  // name that should be relatively unique.
  const processingFileName = fileName === '<input>' ? `${fileKey}.messages.js` : fileName;

  processDefinitionsFile(processingFileName, content, { processTranslations: false });
  /** @type {Record<string, IntlDiagnostic[]>} */
  const validations = {};
  for (const diagnostic of database.validateMessages()) {
    if (diagnostic.file === processingFileName) {
      (validations[diagnostic.key] ??= []).push(diagnostic);
    }
  }

  FILE_VALIDATIONS.set(fileKey, validations);
  return validations;
}

/**
 * Visit all Message definitions in the file, query the native diagnostics generated for the file,
 * reporting the ones that match the given predicate on each definition's value node respectively.
 *
 * @param {RuleContext} context
 * @param {(diagnostic: IntlDiagnostic) => boolean} predicate
 * @returns {RuleListener}
 */
function traverseAndReportMatchingNativeValidations(context, predicate) {
  if (!isDefinitionsFile(context.filename, context.sourceCode.text)) return {};
  const validations = processAndValidateNative(context.filename, context.sourceCode.text);

  return traverseMessageDefinitions(context, (definition, value, _definition, name) => {
    if (name == null) return;
    const diagnostics = validations[name];
    if (diagnostics == null) return;

    for (const diagnostic of diagnostics) {
      if (!predicate(diagnostic)) continue;

      context.report({
        node: value,
        message: diagnostic.description,
      });
    }
  });
}

module.exports = {
  traverseAndReportMatchingNativeValidations,
};
