import { fs, path } from 'zx';
import { REPO_ROOT } from './constants.js';
import * as toml from 'smol-toml';
import assert from 'node:assert';

/**
 * @returns {Promise<string>}
 */
export async function getCargoWorkspaceVersion() {
  const content = await fs.readFile(path.join(REPO_ROOT, 'Cargo.toml'), 'utf8');
  const cargo = toml.parse(content);
  const version = cargo?.workspace?.['package']?.['version'];
  assert(version != null, 'Cargo workspace does not define a version number');
  return version;
}

/**
 * Write the given version to the workspace Cargo.toml file, including for all workspace
 * dependencies.
 *
 * @param {string} newVersion
 * @returns {Promise<void>}
 */
export async function updateCargoWorkspaceVersions(newVersion) {
  const cargoPath = path.join(REPO_ROOT, 'Cargo.toml');
  let content = await fs.readFile(cargoPath, 'utf8');
  content = content.replace(/^version\s+=\s+"[\d.]+"$/m, `version = "${newVersion}"`);
  content = content.replaceAll(
    /\{ version = "=[\d.]+", path =/gm,
    `{ version = "=${newVersion}", path =`,
  );
  await fs.writeFile(cargoPath, content, 'utf8');
}
