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
export async function getWorkspacePackages(options = {}) {
  if (options.cacheOk && WORKSPACE_PACKAGES_CACHE != null) return WORKSPACE_PACKAGES_CACHE;

  const result = await $({ cwd: REPO_ROOT })`pnpm m ls --depth=1 --json`;
  const packages = JSON.parse(result.stdout);
  WORKSPACE_PACKAGES_CACHE = packages.reduce((acc, pack) => {
    acc[pack.name] = pack;
    return acc;
  }, {});
  return WORKSPACE_PACKAGES_CACHE;
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
