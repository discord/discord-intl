/* eslint-disable no-console */
// @ts-check

const { IntlMessagesDatabase, IntlCompiledMessageFormat } = require('..');
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
  let files = database.findAllMessagesFiles(['./data/input'], 'en-US');
  database.processAllMessagesFiles(files);
});

bench('get a message', () => {
  // console.log(
  //   util.inspect(database.getMessage('DISCORD'), {
  //     depth: null,
  //   }),
  // );
});

bench('get source file', () => {
  const source = database.getSourceFileMessageValues(SOURCE_FILES[0]);
  // console.log(Object.entries(source).map(([key, value]) => [key, value?.raw]));
});

bench('export translations', () => {
  const files = database.exportTranslations('messages.jsona');
});

bench('validate', () => {
  const validations = database.validateMessages();
  console.log(
    util.inspect(
      validations.filter((d) => d.severity === 'error'),
      { depth: null },
    ),
  );
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
        for (const [, message] of Object.entries(data)) {
          hydrateFormatJsAst(message);
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
