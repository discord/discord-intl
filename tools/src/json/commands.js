import { Command, Option } from 'commander';

import { npmPublish, npmPublishCommand } from '../npm.js';
import { pnpm } from '../pnpm.js';
import { buildNapiPackage, NAPI_TARGET_MAP } from '../napi.js';
import { checkAllVersionsEqual, getPackageFamily, versionCommand } from '../versioning.js';
import { hostPlatform } from '../util/platform.js';
import { NPM_PACKAGES } from '../constants.js';

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

export default async function () {
  const dbPackage = await pnpm.getPackage(NPM_PACKAGES.JSON_PARSER);
  const dbFamily = await getPackageFamily(dbPackage);

  const group = new Command('json')
    .aliases(['intl-flat-json-parser'])
    .description('Operate on the intl_flat_json_parser crate/package');

  group
    .command('build')
    .description('Build the intl_flat_json_parser native Node extension')
    .addOption(buildTargetOption())
    .action(async ({ target }) => {
      await buildNapiPackage('intl-flat-json-parser', dbPackage, target);
    });

  group.addCommand(versionCommand('version', dbPackage, dbFamily));
  group.addCommand(npmPublishCommand('publish-root', dbPackage));
  group.addCommand(
    npmPublishCommand('publish-target')
      .description('Publish a platform-specific package for intl-flat-json-parser to npm')
      .addOption(buildTargetOption())
      .action(async (options) => {
        const targetPackage = await pnpm.getPackage(
          `@discord/intl-flat-json-parser-${options.target}`,
        );
        await npmPublish(targetPackage, options);
      }),
  );
  group.addCommand(
    npmPublishCommand('publish-all')
      .description(
        'Publish all packages under intl-flat-json-parser. Prefer this command in most situations',
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
