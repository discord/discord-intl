import assert from 'node:assert/strict';
import test from 'node:test';
import { rewriteCargoWorkspaceVersions } from './cargo.js';

const manifest = `[workspace.package]
version  = '0.33.3' # keep formatting
[workspace.dependencies]
first = { version = "=0.33.3", path = "first" }
middle = { default-features = false, version = "=0.33.3", path = "middle" }
last = { path = "last", version = '=0.33.3' }
no_path = { default-features = false, version = "=0.33.3" }
workspace_key = { workspace = true, version = "=0.33.3" }
unpinned = { version = "0.33.3" }
shorthand = "=0.33.3"
external = { version = "=0.33.3" }
alias = { version = "=0.33.3", package = "first" }
# version = "=0.33.3"
[workspace.dependencies.table_form]
default-features = false
version = "=0.33.3"
path = "table_form"
[workspace.metadata]
version = "=0.33.3"
text = 'version = "=0.33.3"'
[dependencies.external]
version = "=0.33.3"
`;

const packageNames = new Set([
  'first',
  'middle',
  'last',
  'no_path',
  'workspace_key',
  'unpinned',
  'shorthand',
  'table_form',
]);

test('rewrites workspace versions without changing formatting or unrelated values', () => {
  const expected = manifest
    .replace("version  = '0.33.3'", "version  = '0.34.0'")
    .replaceAll('version = "=0.33.3"', 'version = "=0.34.0"')
    .replace("version = '=0.33.3'", "version = '=0.34.0'")
    .replace('version = "0.33.3"', 'version = "0.34.0"')
    .replace('shorthand = "=0.33.3"', 'shorthand = "=0.34.0"')
    .replace('external = { version = "=0.34.0" }', 'external = { version = "=0.33.3" }')
    .replace('# version = "=0.34.0"', '# version = "=0.33.3"')
    .replace(
      '[workspace.metadata]\nversion = "=0.34.0"',
      '[workspace.metadata]\nversion = "=0.33.3"',
    )
    .replace('text = \'version = "=0.34.0"\'', 'text = \'version = "=0.33.3"\'')
    .replace(
      '[dependencies.external]\nversion = "=0.34.0"',
      '[dependencies.external]\nversion = "=0.33.3"',
    );
  assert.equal(rewriteCargoWorkspaceVersions(manifest, '0.34.0', packageNames), expected);
  assert.equal(
    rewriteCargoWorkspaceVersions(manifest.replaceAll('\n', '\r\n'), '0.34.0', packageNames),
    expected.replaceAll('\n', '\r\n'),
  );
});

test('handles prerelease versions and quoted version keys', () => {
  const source =
    '[workspace.package]\n"version" = "0.34.0-rc.1"\n[workspace.dependencies]\nfoo = { \'version\' = "=0.34.0-rc.1" }\n';
  assert.equal(
    rewriteCargoWorkspaceVersions(source, '0.34.0', new Set(['foo'])),
    source.replaceAll('0.34.0-rc.1', '0.34.0'),
  );
});
