const fs = require('node:fs');
const path = require('node:path');

const intlJson = require('..');

/**
 * @param {string} title
 * @param {() => unknown} callback
 * @param {boolean} log
 */
function bench(title, callback, log = true) {
  const start = performance.now();
  callback();
  const end = performance.now();
  if (log) {
    console.log(title + ': ', end - start);
  }
}

const REPO_ROOT = path.resolve(__dirname, '../../..');
const DATA_DIR = path.resolve(REPO_ROOT, 'crates/intl_message_database/data/input');

/** @type {Record<string, string>} */
const JSON_FILES = {};
bench('pre-reading content to start', () => {
  const files = fs.readdirSync(DATA_DIR, { withFileTypes: true });
  for (const entry of files) {
    if (entry.isDirectory()) continue;
    if (!entry.name.endsWith('.jsona') && !entry.name.endsWith('.json')) continue;

    const filePath = path.join(entry.path, entry.name);
    JSON_FILES[filePath] = fs.readFileSync(filePath, 'utf8');
  }
  console.log(`Found ${Object.keys(JSON_FILES).length} files`);
});

bench('JSON.parse (default)', () => {
  for (const content of Object.values(JSON_FILES)) {
    JSON.parse(content);
  }
});
bench('JSON.parse (mapped values)', () => {
  for (const content of Object.values(JSON_FILES)) {
    const result = [];
    const messages = JSON.parse(content);
    for (const [key, message] of Object.entries(messages)) {
      result.push({ key, value: message, position: undefined });
    }
  }
});

bench('intlJson.parseJson', () => {
  for (const content of Object.values(JSON_FILES)) {
    intlJson.parseJson(content);
  }
});

bench('intlJson.parseJsonFile', () => {
  for (const filePath of Object.keys(JSON_FILES)) {
    intlJson.parseJsonFile(filePath);
  }
});
