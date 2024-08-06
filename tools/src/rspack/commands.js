import { Command } from 'commander';

import { npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { versionCommand } from '../versioning.js';

export default async function () {
  const pack = await getPackage('@discord/rspack-intl-loader');

  const group = new Command('rspack')
    .aliases(['rspack-intl-loader'])
    .description('Operate on the rspack-intl-loader package');

  group.addCommand(npmPublishCommand('publish', pack));
  group.addCommand(versionCommand('version', pack));

  return group;
}
