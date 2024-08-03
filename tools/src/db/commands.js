import { Command, Option } from 'commander';
import { $ } from 'zx';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { getPackage } from '../pnpm.js';
import { buildNapiPackage, NAPI_TARGET_MAP } from '../napi.js';

const targetOption = new Option('--target <target>')
  .choices(Object.keys(NAPI_TARGET_MAP))
  .makeOptionMandatory(true);

export default async function () {
  const dbPackage = await getPackage('@discord/intl-message-database');

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

  group.addCommand(
    npmPublishCommand(dbPackage)
      .addOption(targetOption)
      .description('Publish a platform-specific package for intl-message-database to npm')
      .action(async (options) => {
        const targetPackage = await getPackage(`@discord/intl-message-database-${options.target}`);
        await npmPublish(targetPackage, options);
      }),
  );

  group.addCommand(
    npmPublishCommand(dbPackage, {
      commandName: 'publish-root',
    }).description('Publish the root intl-message-database package to NPM'),
  );

  return group;
}
