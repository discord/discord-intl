import { $ } from 'zx';
import { Command, Option } from 'commander';

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
    .option(
      '--tag',
      'Tag to additionally apply to the published package. Defaults to `latest`.',
      'latest',
    )
    .option('--provenance', 'Use provenance when publishing the package.')
    .addOption(
      new Option('--access <access>', 'Whether this is publishing a public or private package.')
        .default('public')
        .choices(['public', 'restricted']),
    )
    .description('Publish this package to npm')
    .action(async (options) => {
      await npmPublish(pack, options);
    });
}

/**
 * Run `pnpm publish` with the given arguments.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {{
 *   dryRun?: boolean,
 *   access?: 'public' | 'restricted',
 *   useProvenance?: boolean,
 * }} options
 */
export async function npmPublish(pack, { dryRun, access, useProvenance }) {
  const publishArgs = [
    dryRun ? '--dry-run' : undefined,
    access != null ? `--access=${access}` : undefined,
    // CI setup will create an `.npmrc` file and also modify the `rust-toolchain.toml`, meaning the
    // git state won't be clean, which is _required_ for publishing to npm by default. So we have to
    // explicitly disable that check. Would really rather not do this to enforce that no other git
    // changes leak into releases, but oh well for now.
    process.env.CI === 'true' ? '--no-git-checks' : undefined,
    useProvenance ? '--provenance' : undefined,
  ].filter(Boolean);

  await $({
    cwd: pack.path,
    stdio: 'inherit',
  })`pnpm publish ${publishArgs.join(' ')}`;
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
