import { getPackage } from './pnpm.js';
import { Command } from 'commander';
import { npmPublishCommand } from './npm.js';
import { versionCommand } from './versioning.js';
import { $ } from 'zx';

/**
 * Generate a base set of commands for managing a JavaScript package, including versioning and
 * publishing to npm.
 *
 * @param {string} name
 * @param {{
 *   aliases?: string[],
 *   build?: boolean
 * }=} options
 * @returns {Promise<Command>}
 */
export async function createJsPackageCommands(name, options) {
  const { build = false, aliases = [] } = options;
  const pack = await getPackage('@discord/' + name);

  const group = new Command(name).aliases(aliases).description(`Operate on the ${name} package`);
  group.addCommand(npmPublishCommand('publish', pack));
  group.addCommand(versionCommand('version', pack));

  if (build) {
    group
      .command('build')
      .description(`Build the ${name} package`)
      .action(async () => {
        await $({ cwd: pack.path, stdio: 'inherit' })`pnpm build`;
      });
  }

  return group;
}
