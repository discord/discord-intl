/** @typedef {import('eslint').Rule.RuleListener} RuleListener */
/** @typedef {import('eslint').Rule.RuleContext} RuleContext */
/** @typedef {import('eslint').Rule.ReportDescriptor} ReportDescriptor */
/** @typedef {import('eslint').Rule.ReportDescriptorLocation} ReportDescriptorLocation */
/** @typedef {import('eslint').AST.SourceLocation} SourceLocation */
/** @typedef {import('eslint').SourceCode} SourceCode */
/** @typedef {import("@discord/intl-loader-core/types").IntlDiagnostic} IntlDiagnostic */

const crypto = require('node:crypto');
const {
  processDefinitionsFile,
  database,
  processAllMessagesFiles,
  findAllMessagesFiles,
} = require('@discord/intl-loader-core');
const { traverseMessageDefinitions, isDefinitionsFile } = require('./traverse');
const diagnostics_channel = require('node:diagnostics_channel');
/** @type {Map<SourceCode, Record<string, IntlDiagnostic[]>>} */
const FILE_VALIDATIONS = new Map();

let isInitialized = false;

/**
 * @param {string} directory
 */
function ensureInitialized(directory) {
  if (isInitialized) return;

  processAllMessagesFiles(findAllMessagesFiles([directory]));
  isInitialized = true;
}

/**
 * Process the given file as a definitions file, run the native `validateMessages` function on the
 * database, and store all of the resulting validations.
 *
 * This function will only process a given file once. All future calls will return the cached
 * validations, allowing them to be reused across multiple rules without wasting work validating
 * the same messages multiple times.
 *
 * @param {SourceCode} sourceCode
 * @param {string} fileName
 * @param {string} content
 * @return {Record<string, IntlDiagnostic[]>}
 */
function processAndValidateNative(sourceCode, fileName, content) {
  const fileKey = crypto.hash('sha1', content);
  const existing = FILE_VALIDATIONS.get(sourceCode);
  if (existing != null) return existing;

  // ESLint will use a fake filename for dynamic input. in this case, we need to tell the
  // native database that this is a JS message definition _somehow_, so we'll make a random
  // name that should be relatively unique.
  const processingFileName = fileName === '<input>' ? `${fileKey}.messages.js` : fileName;
  const processResult = processDefinitionsFile(processingFileName, content, {
    processTranslations: false,
  });

  /** @type {Record<string, IntlDiagnostic[]>} */
  const validations = {};

  if (!processResult.succeeded) {
    for (const error of processResult.errors) {
      const key = error.key ?? 'file';
      (validations[key] ??= []).push({
        name: 'Processing::' + error.name,
        description: error.message,
        key: error.key ?? 'file',
        file: error.file ?? processingFileName,
        messageLine: error.line ?? 0,
        messageCol: error.col ?? 0,
        start: 0,
        end: content.length,
        locale: error.locale ?? 'definition',
        category: 'correctness',
        fixes: [],
      });
    }
  }

  for (const diagnostic of database.validateMessages()) {
    if (diagnostic.file === processingFileName) {
      (validations[diagnostic.key] ??= []).push(diagnostic);
    }
  }

  FILE_VALIDATIONS.set(sourceCode, validations);
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

  const projectDirectory = context.settings['intl']?.['projectDirectory'] ?? context.cwd;
  ensureInitialized(projectDirectory);
  const validations = processAndValidateNative(
    context.sourceCode,
    context.filename,
    context.sourceCode.text,
  );

  /**
   * @param {IntlDiagnostic} diagnostic
   * @returns {SourceLocation}
   */
  function reportedSpan(diagnostic) {
    const messageOffset = context.sourceCode.getIndexFromLoc({
      line: diagnostic.messageLine,
      column: diagnostic.messageCol,
    });

    // Sometimes we get a diagnostic pointing to a different file than the
    // current source code. Not totally sure why for now, so lets just noisily
    // log the diagnostic to be able to investigate further when it happens.
    const textLen = context.sourceCode.text.length;
    if (
      diagnostic.file !== context.physicalFilename ||
      messageOffset + diagnostic.start > textLen ||
      messageOffset + diagnostic.end > textLen
    ) {
      console.error('Invalid diagnostic span from intl vs eslint:');
      console.log('ESLint source filename:', context.filename);
      console.log(diagnostic);
      return {
        start: context.sourceCode.getLocFromIndex(1),
        end: context.sourceCode.getLocFromIndex(1),
      };
    }

    return {
      start: context.sourceCode.getLocFromIndex(messageOffset + diagnostic.start),
      end: context.sourceCode.getLocFromIndex(messageOffset + diagnostic.end),
    };
  }

  return traverseMessageDefinitions(context, (_property, value, _definition, name) => {
    if (name == null) return;
    const diagnostics = validations[name];
    if (diagnostics == null) return;

    for (const diagnostic of diagnostics) {
      if (!predicate(diagnostic)) continue;

      /** @type {ReportDescriptor} */
      const report = {
        loc: reportedSpan(diagnostic),
        message: diagnostic.description,
      };
      if (diagnostic.fixes.length > 0 && value.range != null) {
        // Add 1 because ESLint 8 works on 1-based column indices.
        const valueStart = value.range[0] + 1;
        report.fix = (fixer) => {
          return diagnostic.fixes.map((fix) =>
            fixer.replaceTextRange([valueStart + fix.start, valueStart + fix.end], fix.replacement),
          );
        };

        const suggestableFixes = diagnostic.fixes.filter((fix) => fix.message != null);
        if (suggestableFixes.length > 0 && value.range != null) {
          const valueStart = value.range[0];

          report.suggest = suggestableFixes.map((fix) => ({
            desc: /** @type {string} */ (fix.message),
            fix: (fixer) =>
              fixer.replaceTextRange(
                [valueStart + fix.start, valueStart + fix.end],
                fix.replacement,
              ),
          }));
        }
      }

      context.report(report);
    }
  });
}

module.exports = {
  traverseAndReportMatchingNativeValidations,
};
