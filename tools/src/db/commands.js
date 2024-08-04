import { Argument, Command, Option } from 'commander';
import semver from 'semver';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { buildNapiPackage, NAPI_TARGET_MAP } from '../napi.js';
import { bumpAllVersions, checkAllVersionsEqual, getPackageFamily } from './versioning.js';

const targetOption = new Option(
  '--target <target>',
  'Which platform package to build for natively.',
)
  .choices(Object.keys(NAPI_TARGET_MAP))
  .makeOptionMandatory(true);

const DB_PACKAGE_NAME = '@discord/intl-message-database';

export default async function () {
  const dbPackage = await getPackage(DB_PACKAGE_NAME);

  const group = new Command('db')
    .aliases(['intl-message-database'])
    .description('Operate on the intl_message_database crate/package');

  group
    .command('build')
    .description('Build the intl_message_database native Node extension')
    .addOption(targetOption)
    .action(async ({ target }) => {
      await buildNapiPackage(dbPackage, target);
    });

  group
    .command('version')
    .description('Bump the version of all packages around intl-message-database.')
    .addArgument(
      new Argument('<level>', 'Which level of version to bump').choices(
        ['rc', 'canary', 'release', 'set'].concat(semver.RELEASE_TYPES.concat()),
      ),
    )
    .argument(
      '[explicit]',
      'When `level` is `set`, this explicit version will be applied to all packages',
    )
    .action(async (level, explicit) => {
      if (level === 'set') {
        level = { explicit };
      }
      await bumpAllVersions(dbPackage, level);
    });

  group
    .command('version-check')
    .description('Checks that all included packages are currently set to the same version.')
    .action(async () => {
      const allEqual = await checkAllVersionsEqual(dbPackage);
      if (!allEqual) {
        process.exit(1);
      }
    });

  group.addCommand(
    npmPublishCommand('publish-target')
      .description('Publish a platform-specific package for intl-message-database to npm')
      .addOption(targetOption)
      .action(async (options) => {
        const targetPackage = await getPackage(`@discord/intl-message-database-${options.target}`);
        await npmPublish(targetPackage, options);
      }),
  );

  group.addCommand(
    npmPublishCommand('publish-all')
      .description(
        'Publish all packages under intl-message-database. Prefer this command in most situations',
      )
      .action(async (options) => {
        await checkAllVersionsEqual(dbPackage);
        for (const pack of await getPackageFamily(dbPackage)) {
          await npmPublish(pack, options);
        }
      }),
  );

  group.addCommand(
    npmPublishCommand('publish-root').action(
      async (options) => await npmPublish(dbPackage, options),
    ),
  );

  return group;
}
