import { Command, Option } from 'commander';
import { $ } from 'zx';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { TARGET_PACKAGES } from './index.js';
import { getPackage } from '../pnpm.js';
import { buildNapiPackage } from '../napi.js';

const targetOption = new Option('--target <target>')
  .choices(Object.keys(TARGET_PACKAGES))
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
    npmPublishCommand('')
      .addOption(targetOption)
      .description('Publish a platform-specific package for intl-message-database to npm')
      .action(async (options) => {
        const targetPackage = await getPackage(`@discord/intl-message-database-${options.target}`);
        const executor = $({
          cwd: targetPackage.path,
          stdio: 'inherit',
        });
        await npmPublish(executor, options);
      }),
  );

  group.addCommand(
    npmPublishCommand(dbPackage.path, {
      commandName: 'publish-root',
    }).description('Publish the root intl-message-database package to NPM'),
  );

  return group;
}
