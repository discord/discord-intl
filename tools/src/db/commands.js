import path from 'node:path';

import { Command, Option } from 'commander';
import { $ } from 'zx';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { buildNodeExtension, TARGET_PACKAGES } from './index.js';

const targetOption = new Option('--target <target>')
  .choices(Object.keys(TARGET_PACKAGES))
  .makeOptionMandatory(true);

export default function () {
  const group = new Command('db').description('Operate on the intl_message_database crate/package');

  group
    .command('build')
    .description('Build the intl_message_database native Node extension')
    .addOption(targetOption)
    .action(async ({ target }) => {
      await buildNodeExtension(target);
    });

  group.addCommand(
    npmPublishCommand('intl_message_database')
      .addOption(targetOption)
      .description('Publish a platform-specific package for intl-message-database to npm')
      .action(async (options) => {
        const executor = $({
          cwd: path.resolve(`intl_message_database/npm/${options.target}`),
          stdio: 'inherit',
        });
        await npmPublish(executor, options);
      }),
  );

  group.addCommand(
    npmPublishCommand('intl_message_database', {
      commandName: 'publish-root',
    }).description('Publish the root intl-message-database package to NPM'),
  );

  return group;
}
