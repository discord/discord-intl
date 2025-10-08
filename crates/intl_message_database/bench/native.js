/* eslint-disable no-console */
// @ts-check

const {
  IntlMessagesDatabase,
  IntlCompiledMessageFormat,
  IntlDatabaseInsertStrategy,
} = require('..');
const fs = require('node:fs');
const path = require('node:path');
const util = require('node:util');
const { bench, locales: allLocales } = require('./util');
const { hydrateFormatJsAst, compressFormatJsToAst } = require('@discord/intl-ast');

const locales = allLocales;

const database = new IntlMessagesDatabase();

const COMPILATION_FORMAT = /** @type {IntlCompiledMessageFormat} */ (
  IntlCompiledMessageFormat.KeylessJson
);

const SOURCE_FILES = [
  './data/input/en-US.messages.js',
  './data/input/untranslated.messages.js',
  './data/input/international.messages.js',
];

bench('processing', () => {
  let p = './data/input';
  let files = database.findAllMessagesFiles([p], 'en-US');
  database.processAllMessagesFiles(files, IntlDatabaseInsertStrategy.Create);
});

const sourceFile = database.getSourceFile(SOURCE_FILES[0]);

bench('get a message', () => {
  const message = database.getMessage(sourceFile.messageKeys[0]);
  // console.log(util.inspect(message, { depth: null }));
});

bench('get source file', () => {
  const sourceFile = database.getSourceFile(SOURCE_FILES[0]);
  // console.dir(sourceFile, { depth: 1 });
});

bench('export translations', () => {
  const files = database.exportTranslations('messages.jsona');
});

bench('validate', () => {
  const validations = database.validateMessages();
  const errors = validations.filter((d) => d.severity === 'error');
  console.log(errors.length, ' error diagnostics of ', validations.length, ' total');
  // console.log(util.inspect(errors, { depth: null }));
});

bench('generate types', () => {
  database.generateTypes(SOURCE_FILES[0], './data/output/generated.d.ts');
});

/**
 * @param {IntlCompiledMessageFormat} format
 */
function getPrecompileFormat(format) {
  switch (format) {
    case IntlCompiledMessageFormat.Json:
      return 'json';
    case IntlCompiledMessageFormat.KeylessJson:
      return 'keyless';
  }
}

bench(`precompile (${getPrecompileFormat(COMPILATION_FORMAT)})`, () => {
  const locales = database.getKnownLocales();
  for (const locale of locales) {
    database.precompile(SOURCE_FILES[0], locale, `./data/output/${locale}.json`, {
      format: COMPILATION_FORMAT,
      bundleSecrets: true,
    });
  }
});

/** @type {Record<string, any>} */
const COMPILED_FILES = {};

bench('read compiled files', () => {
  const files = fs.readdirSync('./data/output/');
  for (const file of files) {
    if (path.extname(file) === '.json') {
      COMPILED_FILES[path.basename(file)] = fs.readFileSync(`./data/output/${file}`).toString();
    }
  }
});

bench('parse json', () => {
  for (const [locale, data] of Object.entries(COMPILED_FILES)) {
    COMPILED_FILES[locale] = JSON.parse(data);
  }
});

switch (COMPILATION_FORMAT) {
  case IntlCompiledMessageFormat.KeylessJson:
    bench('hydrate json', () => {
      for (const [, data] of Object.entries(COMPILED_FILES)) {
        for (const [key, message] of Object.entries(data)) {
          try {
            hydrateFormatJsAst(message);
          } catch (e) {
            console.log(`Failed to parse ${key}:`, util.inspect(message, { depth: null }));
            throw e;
          }
        }
      }
    });
    break;

  case IntlCompiledMessageFormat.Json:
    bench('compress json', () => {
      for (const [, data] of Object.entries(COMPILED_FILES)) {
        for (const [, message] of Object.entries(data)) {
          compressFormatJsToAst(message);
        }
      }
    });
    break;
}
