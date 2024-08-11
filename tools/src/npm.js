import { Command, Option } from 'commander';
import { $ } from 'zx';

import { pnpm } from './pnpm.js';

/**
 * Create a bare `publish` command, with all of the flags used for passing to `npmPublish`
 * separately. This command does not include any action by default, but the basic action can be
 * implemented like:
 *
 * ```typescript
 * npmPublishCommand().action((options) => npmPublish(somePackage, options));
 * ```
 *
 * @param {string=} commandName
 * @param {import('./pnpm.js').PnpmPackage=} pack
 * @returns {Command}
 */
export function npmPublishCommand(commandName = 'publish', pack) {
  const command = new Command(commandName)
    .option('--dry-run', "Don't actually publish the package")
    .option(
      '--tag <tag>',
      'Tag to additionally apply to the published package. Defaults to `latest`.',
      'latest',
    )
    .option('--provenance', 'Use provenance when publishing the package.')
    .option(
      '--skip-existing',
      'Skip publishing packages where the current version has already been published.',
    )
    .option('--no-git-checks', 'Whether to enforce a clean git state before publishing.')
    .addOption(
      new Option('--access <access>', 'Whether this is publishing a public or private package.')
        .default('public')
        .choices(['public', 'restricted']),
    )
    .description('Publish this package to npm');

  if (pack != null) {
    command.action(async (options) => await npmPublish(pack, options));
  }

  return command;
}

/**
 * Run `pnpm publish` with the given arguments.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {{
 *   dryRun?: boolean,
 *   access?: 'public' | 'restricted',
 *   useProvenance?: boolean,
 *   gitChecks?: boolean
 *   tag?: string,
 *   skipExisting?: boolean,
 * }} options
 */
export async function npmPublish(pack, options) {
  const { dryRun, access, useProvenance, gitChecks, tag, skipExisting = false } = options;

  // Only check the existing package versions if the caller is allowing it to be skipped. Otherwise,
  // NPM will error out on a version conflict, which we want to propagate directly.
  if (skipExisting && (await isVersionAlreadyPublished(pack))) {
    console.log('Checking if package already exists');
    return;
  }

  const publishArgs = [
    dryRun ? '--dry-run' : undefined,
    access != null ? `--access=${access}` : undefined,
    // CI setup will create an `.npmrc` file and also modify the `rust-toolchain.toml`, meaning the
    // git state won't be clean, which is _required_ for publishing to npm by default. So we have to
    // explicitly disable that check. Would really rather not do this to enforce that no other git
    // changes leak into releases, but oh well for now.
    gitChecks ? '--git-checks' : '--no-git-checks',
    tag ? `--tag=${tag}` : undefined,
  ].filter(Boolean);

  await $({
    cwd: pack.path,
    stdio: 'inherit',
    env: {
      ...process.env,
      // The `--provenance` flag doesn't seem to forward properly with pnpm, so use this env var
      // instead to pass it through.
      // See: https://github.com/pnpm/pnpm/issues/6607
      NPM_CONFIG_PROVENANCE: useProvenance ? 'true' : 'false',
    },
  })`pnpm publish ${publishArgs}`;
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
