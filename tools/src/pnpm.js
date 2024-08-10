import fs from 'node:fs/promises';
import path from 'node:path';

import { $ } from 'zx';

import { REPO_ROOT } from './constants.js';

/**
 * @typedef PnpmPackage
 * @property {string} name
 * @property {string} version
 * @property {string} path
 * @property {boolean} private
 * @property {Record<string, object>} dependencies
 * @property {Record<string, object>} devDependencies
 * @property {Record<string, object>} peerDependencies
 * @property {Record<string, object>} optionalDependencies
 * @property {Record<string, object>} unsavedDependencies
 */

let WORKSPACE_PACKAGES_CACHE = undefined;

/**
 * Return a record of all JS packages contained in the workspace, along with meta information about
 * each one.
 *
 * @param {{cacheOk?: boolean}} options
 * @returns {Promise<Record<string, PnpmPackage>>}
 */
export async function getWorkspacePackages({ cacheOk = true } = {}) {
  if (cacheOk && WORKSPACE_PACKAGES_CACHE != null) return WORKSPACE_PACKAGES_CACHE;

  const result = await $({ cwd: REPO_ROOT })`pnpm m ls --depth=1 --json`;
  const packages = JSON.parse(result.stdout);
  WORKSPACE_PACKAGES_CACHE = packages.reduce((acc, pack) => {
    acc[pack.name] = pack;
    return acc;
  }, {});
  return WORKSPACE_PACKAGES_CACHE;
}

export async function getPublicPackages() {
  const workspace = Object.values(await getWorkspacePackages());
  return workspace.filter((pack) => pack.private === false);
}

/**
 * Return a single package entry from the pnpm workspace.
 *
 * @param {string} packageName
 * @returns {Promise<PnpmPackage>}
 */
export async function getPackage(packageName) {
  const packages = await getWorkspacePackages();
  return packages[packageName];
}

/**
 * Return the package json content for the given package.
 * @param {PnpmPackage} pack
 * @returns {Promise<object>}
 */
export async function getPackageJson(pack) {
  const packageJsonPath = path.resolve(pack.path, 'package.json');
  const content = await fs.readFile(packageJsonPath);
  return JSON.parse(content.toString());
}

/**
 * Parse the existing package.json for the given package, send it to the provided callback, then
 * write the result back to the package.json file.
 *
 * @param {PnpmPackage} pack
 * @param {<T>(json: any) => T | Promise<T>} mutator
 * @returns {Promise<object>}
 */
export async function updatePackageJson(pack, mutator) {
  const content = await getPackageJson(pack);
  const updated = await mutator(content);
  await fs.writeFile(path.resolve(pack.path, 'package.json'), JSON.stringify(updated, null, 2));
  return updated;
}

export const pnpm = {
  getPublicPackages,
};
