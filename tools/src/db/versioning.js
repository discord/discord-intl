import { getWorkspacePackages, updatePackageJson } from '../pnpm.js';
import semver from 'semver';
import { $ } from 'zx';

/**
 * @typedef {import('../pnpm.js').PnpmPackage} PnpmPackage
 * @typedef {import('semver').ReleaseType | 'rc' | 'release' | 'canary' | {explicit: string}} VersionSpec
 */

/**
 * Get all of the packages in the workspace that are related to intl-message-database, since they
 * should all be named `<prefix>-<extension>`. The resulting array includes both the root package
 * _and_ all of the sub-packages.
 *
 * @param {PnpmPackage} dbPackage
 * @returns {Promise<PnpmPackage[]>}
 */
export async function getPackageFamily(dbPackage) {
  return Object.entries(await getWorkspacePackages()).reduce((acc, [name, pack]) => {
    if (name.startsWith(dbPackage.name)) acc.push(pack);
    return acc;
  }, []);
}

/**
 *
 * @param {import('semver').SemVer} baseVersion
 * @param {VersionSpec} level
 * @returns {string}
 */
function applyVersionBump(baseVersion, level) {
  // Release versioning just removes any pre-release tags
  if (level === 'release') {
    return semver.coerce(baseVersion.version, { includePrerelease: false }).version;
  }

  // RC creates a release candidate, which is just a prerelease version bump of whatever the
  // current version is.
  if (level === 'rc') {
    level = 'prerelease';
  }

  if (typeof level === 'object' && 'explicit' in level) {
    return level.explicit;
  }

  switch (level) {
    case 'premajor':
    case 'preminor':
    case 'prepatch':
    case 'prerelease':
      return baseVersion.inc(level, 'rc').version;
    case 'canary':
      return semver.inc(
        baseVersion,
        'prerelease',
        'canary-' + $.sync`git rev-parse --short HEAD`.stdout.trim(),
        false,
      );
    default:
      return baseVersion.inc(level).version;
  }
}

/**
 * Return the parsed SemVer value of the version specified in the package.json of the given package.
 * @param {PnpmPackage} pack
 */
function getPackageVersion(pack) {
  const parsed = semver.coerce(pack.version, {
    includePrerelease: true,
    loose: true,
  });
  if (pack.version !== parsed.version) {
    console.warn(`[semver] Current version ${pack.version} was coerced to ${parsed.version}`);
  }
  return parsed;
}

/**
 * Bump all package versions for intl-message-database packages.
 *
 * @param {PnpmPackage} dbPackage
 * @param {VersionSpec} level
 * @returns {Promise<void>}
 */
export async function bumpAllVersions(dbPackage, level) {
  const rootVersion = getPackageVersion(dbPackage);
  const bumpedVersion = applyVersionBump(rootVersion, level);
  if (bumpedVersion == null) {
    throw new Error(
      `Version bump of level ${JSON.stringify(level)} from ${rootVersion.version} did not create a valid version`,
    );
  }
  console.info(`Bumping all intl-message-database packages to ${bumpedVersion}`);
  for (const pack of await getPackageFamily(dbPackage)) {
    await updatePackageJson(pack, (json) => {
      json.version = bumpedVersion;
      return json;
    });
  }
}

/**
 * Checks that all packages under `dbPackage` are currently set to the same version specifier.
 *
 * @param {PnpmPackage} dbPackage
 * @returns {Promise<boolean>}
 */
export async function checkAllVersionsEqual(dbPackage) {
  const subPackages = await getPackageFamily(dbPackage);
  let allValid = true;
  for (const pack of subPackages) {
    if (pack.version !== dbPackage.version) {
      console.warn(
        `[version-check] ${pack.name}@${pack.version} does not match root version ${dbPackage.version}`,
      );
      allValid = false;
    }
  }

  if (allValid) {
    console.info(`[version-check] All packages have matching versions: ${dbPackage.version}`);
  }
  return allValid;
}
