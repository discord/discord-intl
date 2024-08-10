import semver from 'semver';
import { Argument, Command } from 'commander';

import { git } from './util/git.js';
import { getWorkspacePackages, updatePackageJson } from './pnpm.js';

/**
 * @typedef {import('./pnpm.js').PnpmPackage} PnpmPackage
 * @typedef {import('semver').ReleaseType | 'rc' | 'release' | 'canary' | {explicit: string}} VersionSpec
 */

/**
 * Return a group of commands for managing the version of `basePack`, along with any other packages
 * along with it as part of `packageFamily`. If set, `packageFamily` must also include `basePack`,
 * otherwise it can be omitted entirely.
 *
 * @param {string} groupName
 * @param {PnpmPackage} basePack
 * @param {PnpmPackage[]=} packageFamily
 * @returns {Command}
 */
export function versionCommand(groupName, basePack, packageFamily) {
  const hasFamily = packageFamily?.length > 0;
  const group = new Command(groupName);
  group
    .command('bump')
    .description(
      hasFamily
        ? `Bump the version of all packages around ${basePack.name}`
        : `Bump the version of ${basePack.name}`,
    )
    .addArgument(
      new Argument('<level>', 'Which level of version to bump').choices(
        ['rc', 'canary', 'release', 'set'].concat(semver.RELEASE_TYPES.concat()),
      ),
    )
    .argument(
      '[explicit]',
      'When `level` is `set`, this explicit version will be applied to all packages',
    )
    .action(async (level, explicit) => {
      if (level === 'set') {
        level = { explicit };
      }
      hasFamily
        ? await bumpAllVersions(basePack, packageFamily, level)
        : await bumpVersion(basePack, level);
    });

  group
    .command('show')
    .description(
      hasFamily
        ? `List all version of packages around ${basePack.name}`
        : `Show the current version of ${basePack.name}`,
    )
    .option('--short', 'Print only the raw version from the package.json of the package.')
    .action(({ short }) => {
      /** @type {(pack: PnpmPackage) => void} */
      const log = (pack) => {
        const version = getPackageVersion(pack);
        console.log(`${pack.name}@${version.version} ${short ? '' : `(raw: ${version.raw})`}`);
      };

      if (hasFamily) {
        for (const pack of packageFamily) {
          log(pack);
        }
      } else {
        log(basePack);
      }
    });

  if (hasFamily) {
    group
      .command('check')
      .description('Checks that all included packages are currently set to the same version.')
      .action(() => {
        if (!checkAllVersionsEqual(basePack, packageFamily)) {
          process.exit(1);
        }
      });
  }

  return group;
}

/**
 * Get all of the packages in the workspace that are related to the given base package, since they
 * should all be named `<prefix>-<extension>`. The resulting array includes both the root package
 * _and_ all of the sub-packages.
 *
 * @param {PnpmPackage} basePackage
 * @returns {Promise<PnpmPackage[]>}
 */
export async function getPackageFamily(basePackage) {
  return Object.entries(await getWorkspacePackages()).reduce((acc, [name, pack]) => {
    if (name.startsWith(basePackage.name)) acc.push(pack);
    return acc;
  }, []);
}

/**
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
      const canaryBase = semver.coerce(baseVersion.version, { includePrerelease: false });
      const currentHead = git.currentHead({ short: true });
      return `${canaryBase}-canary.${currentHead}`;
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
 * Bump the version of the given `basePackage`, then apply the same version number to all packages
 * in `packageFamily`. The base package is expected to be part of the `packageFamily` in order to
 * have the bump written to it as well. For single packages, use `bumpVersion` instead.
 *
 * @param {PnpmPackage} basePackage
 * @param {PnpmPackage[]} packageFamily
 * @param {VersionSpec} level
 * @returns {Promise<void>}
 */
export async function bumpAllVersions(basePackage, packageFamily, level) {
  const rootVersion = getPackageVersion(basePackage);
  const bumpedVersion = applyVersionBump(rootVersion, level);
  if (bumpedVersion == null) {
    throw new Error(
      `Version bump of level ${JSON.stringify(level)} from ${rootVersion.version} did not create a valid version`,
    );
  }
  console.info(`Bumping all packages to match ${basePackage.name}: ${bumpedVersion}`);
  for (const pack of packageFamily) {
    await updatePackageJson(pack, (json) => {
      json.version = bumpedVersion;
      return json;
    });
    console.info(`- ${pack.name} now at ${bumpedVersion}`);
  }
}

/**
 * Bump the version of the given `pack` by `level`.
 *
 * @param {PnpmPackage} pack
 * @param {VersionSpec} level
 * @returns {Promise<void>}
 */
export async function bumpVersion(pack, level) {
  await bumpAllVersions(pack, [pack], level);
}

/**
 * Checks that all packages in `packageFamily` have a version specifier that exactly matches the one
 * given in `basePackage`.
 *
 * @param {PnpmPackage} basePackage
 * @param {PnpmPackage[]} packageFamily
 * @returns {boolean}
 */
export function checkAllVersionsEqual(basePackage, packageFamily) {
  let allValid = true;
  for (const pack of packageFamily) {
    if (pack.version !== basePackage.version) {
      console.warn(
        `[version-check] ${pack.name}@${pack.version} does not match root version ${basePackage.version}`,
      );
      allValid = false;
    }
  }

  if (allValid) {
    console.info(`[version-check] All packages have matching versions: ${basePackage.version}`);
  }
  return allValid;
}
