import { Command } from 'commander';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { getPackage, getWorkspacePackages } from '../pnpm.js';
import { checkAllVersionsEqual, versionCommand } from '../versioning.js';

export default async function () {
  const allPackages = Object.values(await getWorkspacePackages());
  const publicPackages = allPackages.filter((pack) => pack.private === false);
  // The db package is the core that all other versions are based off of, so it's used as the
  // single arg, while the rest of the packages are treated as the package family.
  const dbPackage = await getPackage('@discord/intl-message-database');

  const group = new Command('eco')
    .aliases(['ecosystem'])
    .description('Operate on the entire ecosystem of JS packages in the repo.');

  group.addCommand(
    npmPublishCommand('publish-all').action(async (options) => {
      console.info('Ensuring all public packages have matching versions');
      if (!checkAllVersionsEqual(dbPackage, publicPackages)) {
        process.exit(1);
      }

      for (const pack of publicPackages) {
        await npmPublish(pack, options);
      }
    }),
  );
  group.addCommand(versionCommand('version', dbPackage, publicPackages));

  return group;
}
