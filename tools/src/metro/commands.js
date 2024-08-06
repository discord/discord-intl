import { Command } from 'commander';

import { npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { versionCommand } from '../versioning.js';

export default async function () {
  const pack = await getPackage('@discord/metro-intl-transformer');

  const group = new Command('metro')
    .aliases(['metro-intl-transformer'])
    .description('Operate on the metro-intl-transformer package');

  group.addCommand(npmPublishCommand('publish', pack));
  group.addCommand(versionCommand('version', pack));

  return group;
}
