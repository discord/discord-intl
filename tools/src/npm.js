import path from 'node:path';

import { $ } from 'zx';
import { Command } from 'commander';

/**
 * Create a `publish` command, used to publish an NPM package from the configured directory.
 *
 * Additional options are added to the command for doing dry runs and other validation.
 *
 * @param {string} packageDirectory
 * @param {{
 *   commandName?: string,
 * }=} options
 * @returns {Command}
 */
export function npmPublishCommand(packageDirectory, { commandName = 'publish' } = {}) {
  return new Command(commandName)
    .option('--dry-run', "Don't actually publish the package")
    .description('Publish this package to npm')
    .action(async ({ dryRun }) => {
      const executor = $({ cwd: path.resolve(packageDirectory), stdio: 'inherit' });
      await npmPublish(executor, { dryRun });
    });
}

/**
 * Run `pnpm publish` using the given executor.
 * @param {import('zx').Shell} executor
 * @param {{
 *   dryRun?: boolean,
 *   access?: boolean,
 * }} options
 */
export async function npmPublish(executor, { dryRun, access }) {
  const dryRunArg = dryRun ? '--dry-run' : '';
  const accessArg = access != null ? `--access=${access}` : '';
  await executor`pnpm publish ${dryRunArg} ${accessArg}`;
}
