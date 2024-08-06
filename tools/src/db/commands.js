import { Command, Option } from 'commander';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { buildNapiPackage, NAPI_TARGET_MAP } from '../napi.js';
import { checkAllVersionsEqual, getPackageFamily, versionCommand } from '../versioning.js';

const targetOption = new Option(
  '--target <target>',
  'Which platform package to build for natively.',
)
  .choices(Object.keys(NAPI_TARGET_MAP))
  .makeOptionMandatory(true);

const DB_PACKAGE_NAME = '@discord/intl-message-database';

export default async function () {
  const dbPackage = await getPackage(DB_PACKAGE_NAME);
  const dbFamily = await getPackageFamily(dbPackage);

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

  group.addCommand(versionCommand('version', dbPackage, dbFamily));

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
        if (!checkAllVersionsEqual(dbPackage, dbFamily)) {
          process.exit(1);
        }

        for (const pack of dbFamily) {
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
