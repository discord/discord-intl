import { Command, Option } from 'commander';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { pnpm } from '../pnpm.js';
import { buildNapiPackage, NAPI_TARGET_MAP } from '../napi.js';
import { checkAllVersionsEqual, getPackageFamily, versionCommand } from '../versioning.js';
import { hostPlatform } from '../util/platform.js';

/**
 * Return a new option, `--target`, that specifies which build target should be used for the parent
 * command.
 *
 * Target names are the platform-package names, not the actual host triple from the system. More
 * information about the host can be looked up using `NAPI_TARGET_MAP` or the `platform` utils.
 *
 * `local` can also be given to automatically determine and use the host platform as the target.
 *
 * @returns {Option}
 */
export function buildTargetOption() {
  return new Option('--target <target>', 'Which platform package to build for natively.')
    .choices(Object.keys(NAPI_TARGET_MAP).concat(['local']))
    .argParser((target) => (target === 'local' ? hostPlatform.target : target))
    .makeOptionMandatory(true);
}

const DB_PACKAGE_NAME = '@discord/intl-message-database';

export default async function () {
  const dbPackage = await pnpm.getPackage(DB_PACKAGE_NAME);
  const dbFamily = await getPackageFamily(dbPackage);

  const group = new Command('db')
    .aliases(['intl-message-database'])
    .description('Operate on the intl_message_database crate/package');

  group
    .command('build')
    .description('Build the intl_message_database native Node extension')
    .addOption(buildTargetOption())
    .action(async ({ target }) => {
      await buildNapiPackage(dbPackage, target);
    });

  group
    .command('bench')
    .description('Run the database benchmark in `bench/native.js`')
    .option('--build', 'Rebuild the crate locally before running the bench')
    .action(async ({ build }) => {
      if (build) {
        await buildNapiPackage(dbPackage, hostPlatform.target);
      }
      await pnpm.runScriptInPackage(dbPackage, 'bench:native');
    });

  group.addCommand(versionCommand('version', dbPackage, dbFamily));
  group.addCommand(npmPublishCommand('publish-root', dbPackage));
  group.addCommand(
    npmPublishCommand('publish-target')
      .description('Publish a platform-specific package for intl-message-database to npm')
      .addOption(buildTargetOption())
      .action(async (options) => {
        const targetPackage = await pnpm.getPackage(
          `@discord/intl-message-database-${options.target}`,
        );
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

  return group;
}
