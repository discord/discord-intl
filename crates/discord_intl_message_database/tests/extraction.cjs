const assert = require('node:assert/strict');
const { IntlMessagesDatabase } = require(process.argv[2]);
const db = new IntlMessagesDatabase();
const accepted = db.extractFileContentWithDiagnostics(
  'virtual.js',
  "import {defineMessages} from '@discord/intl'; export default defineMessages({A:`static`});",
);
assert.equal(accepted.complete, true);
assert.equal(accepted.messages[0].text, 'static');
const observed = db.extractFileContentWithDiagnostics(
  'virtual.js',
  "import {defineMessages} from '@discord/intl'; export default defineMessages({A:{message:`text`,description:`ignored`,secret:!0}}); export const meta={secret:true};",
);
assert.equal(observed.complete, false);
assert.equal(observed.messages[0].text, 'text');
assert.equal(observed.messages[0].meta.description, null);
assert.equal(observed.messages[0].meta.secret, false);
assert.equal(observed.meta.secret, true);
assert.equal(observed.diagnostics[0].code, 'IgnoredMetadata');
const unicode = db.extractFileContentWithDiagnostics(
  'virtual.js',
  String.raw`import {defineMessages} from '@discord/intl'; export default defineMessages({A:'\uD800'});`,
);
assert.equal(unicode.messages[0].text, '\uFFFD');
assert.equal(unicode.diagnostics[0].code, 'InvalidUnicode');
const json = db.extractFileContentWithDiagnostics('en.json', String.raw`{"A":"\q"}`);
assert.equal(json.complete, true);
assert.equal(json.messages[0].text, 'q');
assert.equal('raw' in accepted.messages[0], false);
assert.equal(typeof db.getSourceSpelling, 'undefined');
assert.equal(typeof db.dumpAsCsvs, 'function');
