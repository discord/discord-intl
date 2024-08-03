import { Command } from 'commander';
import { $ } from 'zx';

import { npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';

/**
 * @param {import('../pnpm.js').PnpmPackage} pack
 * @returns {Promise<void>}
 */
async function buildWasm(pack) {
  $({ cwd: pack.path, stdio: 'inherit' })`pnpm build`;
}

export default async function () {
  const pack = await getPackage('@discord/swc-intl-message-transformer');

  const group = new Command('swc')
    .aliases(['swc-intl-message-transformer'])
    .description('Operate on the intl_message_database crate/package');

  group
    .command('build')
    .description('Build the swc-intl-message-transformer WASM plugin')
    .action(async () => {
      await buildWasm(pack);
    });

  group.addCommand(npmPublishCommand(pack));

  return group;
}
