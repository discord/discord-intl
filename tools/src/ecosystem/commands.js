import fs from 'node:fs';
import path from 'node:path';

import { Command } from 'commander';
import { $ } from 'zx';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { getPackage, getWorkspacePackages } from '../pnpm.js';
import { checkAllVersionsEqual, versionCommand } from '../versioning.js';
import { REPO_ROOT } from '../constants.js';

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

  group
    .command('local-pack')
    .description(
      'Build all of the packages in the repo and package them up as tarballs that can be required from another project locally.',
    )
    .option(
      '--build,--no-build',
      'Skip building compiled packages. Useful if you are only working on JS-land packages',
      true,
    )
    .action(async ({ build }) => {
      if (build) {
        console.info('[local-pack] Building all compilable packages');
        await Promise.all([
          $`pnpm intl-cli db build --target local`,
          $`pnpm intl-cli swc build`,
          $`pnpm intl-cli runtime build`,
        ]);
      }

      console.log('[local-pack] Building complete. Creating packs:');

      const packsPath = path.resolve(REPO_ROOT, '.local-packs');
      fs.mkdirSync(packsPath, { recursive: true });
      for (const pack of publicPackages) {
        const result = await $({
          cwd: pack.path,
          stdio: 'pipe',
        })`pnpm pack --pack-destination ${packsPath}`;
        const artifactName = result.stdout.trim();
        console.log(`- ${pack.name} -> ${artifactName}`);
      }
    });

  return group;
}
