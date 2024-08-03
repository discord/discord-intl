import path from 'node:path';

import { $ } from 'zx';
import { Command } from 'commander';

/**
 * Create a `publish` command, used to publish an NPM package from the configured directory.
 *
 * Additional options are added to the command for doing dry runs and other validation.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {{
 *   commandName?: string,
 * }=} options
 * @returns {Command}
 */
export function npmPublishCommand(pack, { commandName = 'publish' } = {}) {
  return new Command(commandName)
    .option('--dry-run', "Don't actually publish the package")
    .description('Publish this package to npm')
    .action(async ({ dryRun }) => {
      await npmPublish(pack, { dryRun });
    });
}

/**
 * Run `pnpm publish` using the given executor.
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {{
 *   dryRun?: boolean,
 *   access?: boolean,
 * }} options
 */
export async function npmPublish(pack, { dryRun, access }) {
  const dryRunArg = dryRun ? '--dry-run' : '';
  const accessArg = access != null ? `--access=${access}` : '';
  // setup-node on CI will create a new `.npmrc` with an auth token on it already...which means the
  // git state won't be clean, which is _required_ for publishing to npm by default. So we have to
  // explicitly disable that check. Would really rather not do this to enforce that no other git
  // changes leak into releases, but oh well for now.
  const gitChecksArg = process.env.CI === 'true' ? '--no-git-checks' : '';

  // Avoid even trying to publish a version that already exists.
  if (await isVersionAlreadyPublished(pack)) {
    console.error(
      `${pack.name}@${pack.version} is already published to npm. Refusing to continue publishing`,
    );
    process.exit(1);
  }

  await $({
    cwd: pack.path,
    stdio: 'inherit',
  })`pnpm publish ${dryRunArg} ${accessArg} ${gitChecksArg}`;
}

/**
 * Check if the configured version of a package is already published to NPM.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @returns {Promise<boolean>}
 */
export async function isVersionAlreadyPublished(pack) {
  const response = await $({
    nothrow: true,
    quiet: true,
  })`pnpm view ${pack.name} --json`;

  const packageInfo = JSON.parse(response.stdout);

  // pnpm view will error out if the package doesn't exist on NPM, but if there's a bad exit code
  // and 404 _isn't_ in the output, then it's likely something else that went wrong.
  if (response.exitCode !== 0) {
    if (packageInfo['error']['code'] === 'E404') {
      return false;
    } else {
      throw new Error(`Failed to fetch versions from npm for ${pack.name}`);
    }
  }

  return packageInfo.versions.includes(pack.version);
}
