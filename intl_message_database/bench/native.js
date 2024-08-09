/* eslint-disable no-console */
// @ts-check

const { IntlMessagesDatabase, IntlCompiledMessageFormat } = require('..');
const fs = require('node:fs');
const path = require('node:path');
const util = require('node:util');
const { bench, locales: allLocales } = require('./util');
const { hydrateMessages } = require('./keyless-json');

const locales = allLocales;

const database = new IntlMessagesDatabase();

/** @type {IntlCompiledMessageFormat} */
const COMPILATION_FORMAT = IntlCompiledMessageFormat.KeylessJson;

bench('processing', () => {
  database.processDefinitionsFile('./data/input/en-US.js');
  //   native.processDefinitionsFile(database, './data/input/en-US.untranslated.js');
  //   native.processDefinitionsFile(database, './data/input/international.untranslated.js');

  // // Single threaded:
  // for (const locale of locales) {
  //     native.processTranslationsFile(database, `./data/input/${locale}.jsona`, locale);
  // }

  // Multithreaded:
  /** @type {Record<string, string>} */
  const localeMap = {};
  for (const locale of locales) {
    localeMap[locale] = `./data/input/${locale}.jsona`;
  }

  database.processAllTranslationFiles(localeMap);
});

bench('get a message', () => {
  // console.log(
  util.inspect(
    database.getMessage('GUILD_SETTINGS_AUDIT_LOG_CHANNEL_PERMISSION_OVERRIDES_DENIED'),
    {
      depth: null,
    },
  );
  // );
});

bench('get source file', () => {
  const source = database.getSourceFile('./data/input/en-US.js');
});

bench('validate', () => {
  database.validateMessages();
  //   const allDiagnostics = validations.flatMap((v) => v.diagnostics);
  // console.log(
  //     util.inspect(
  //         allDiagnostics.filter((d) => d.severity === 'Error'),
  //         {depth: null},
  //     ),
  // );
  // console.log(util.inspect(validations, {depth: null}));
});

bench('generate types', () => {
  const paths = database.getAllSourceFilePaths();
  database.generateTypes(paths[0], './data/output/generated.d.ts');
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
    database.precompile(
      './data/input/en-US.js',
      locale,
      `./data/output/${locale}.json`,
      COMPILATION_FORMAT,
    );
  }
});

/** @type {Record<string, any>} */
const COMPILED_FILES = {};

bench('read compiled files', () => {
  const files = fs.readdirSync('./data/output/');
  for (const file of files) {
    if (path.extname(file) === '.json') {
      // if (file === 'en-US.json') {
      COMPILED_FILES[path.basename(file)] = fs.readFileSync(`./data/output/${file}`).toString();
    }
  }
});

bench('parse json', () => {
  for (const [locale, data] of Object.entries(COMPILED_FILES)) {
    COMPILED_FILES[locale] = JSON.parse(data);
  }
});

if (COMPILATION_FORMAT === IntlCompiledMessageFormat.KeylessJson) {
  bench('hydrate json', () => {
    for (const [, data] of Object.entries(COMPILED_FILES)) {
      hydrateMessages(data);
    }
  });
}
