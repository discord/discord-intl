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
  const content = await fs.readFile(cargoPath, 'utf8');
  const cargo = toml.parse(content);
  const packageNames = new Set(
    await Promise.all(
      cargo.workspace.members.map(async (member) => {
        const manifest = await fs.readFile(path.join(REPO_ROOT, member, 'Cargo.toml'), 'utf8');
        return toml.parse(manifest).package.name;
      }),
    ),
  );
  await fs.writeFile(
    cargoPath,
    rewriteCargoWorkspaceVersions(content, newVersion, packageNames),
    'utf8',
  );
}

export function rewriteCargoWorkspaceVersions(content, newVersion, packageNames) {
  const cargo = toml.parse(content);
  const oldVersion = cargo.workspace?.package?.version;
  assert(typeof oldVersion === 'string', 'Cargo workspace does not define a version number');
  const escapedVersion = oldVersion.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const versions = new RegExp(`(["'])(=?${escapedVersion})\\1`, 'g');
  const marker = '__discord_intl_version__';

  return content.replace(versions, (literal, quote, version, offset) => {
    // Probe parsed values to leave comments, unrelated strings, and other tables untouched.
    const probe = toml.parse(
      content.slice(0, offset) + quote + marker + quote + content.slice(offset + literal.length),
    );
    const workspace = probe.workspace;
    const isPackage = workspace?.package?.version === marker;
    const isDependency = Object.entries(workspace?.dependencies ?? {}).some(
      ([name, dependency]) =>
        packageNames.has(dependency?.package ?? name) &&
        (dependency?.version === marker || dependency === marker),
    );
    return isPackage || isDependency
      ? `${quote}${version.startsWith('=') ? '=' : ''}${newVersion}${quote}`
      : literal;
  });
}
