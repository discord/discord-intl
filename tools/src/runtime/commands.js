import { Command } from 'commander';
import { $ } from 'zx';

import { npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { versionCommand } from '../versioning.js';

/**
 * @param {import('../pnpm.js').PnpmPackage} pack
 * @returns {Promise<void>}
 */
async function buildJs(pack) {
  $({ cwd: pack.path, stdio: 'inherit' })`pnpm build`;
}

export default async function () {
  const pack = await getPackage('@discord/intl');

  const group = new Command('runtime').description(
    'Operate on the intl package, the client runtime.',
  );

  group
    .command('build')
    .description('Build the intl runtime package to prepare for release.')
    .action(async () => {
      await buildJs(pack);
    });

  group.addCommand(npmPublishCommand('publish', pack));
  group.addCommand(versionCommand('version', pack));

  return group;
}
