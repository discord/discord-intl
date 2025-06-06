import fs from 'node:fs/promises';
import path from 'node:path';
import { $ } from 'zx';

import { Argument, Command } from 'commander';

import { CRATES, NPM_PACKAGES } from './constants.js';
import { NAPI_TARGET_MAP } from './napi.js';
import { pnpm } from './pnpm.js';

export default async function () {
  const group = new Command('util').description(
    'Random utilities for generating, inspecting, and querying things',
  );

  group
    .command('gen-cjk-ranges')
    .description(
      'Generate CJK range bounds for intl_markdown based on the current Unicode standard',
    )
    .action(async () => {
      await $({
        cwd: CRATES.INTL_MARKDOWN,
        stdio: ['inherit', 'inherit', 'ignore'],
      })`node --experimental-strip-types ./scripts/cjk-ranges.ts -l rust`;
    });

  group
    .command('package-triple')
    .addArgument(
      new Argument('<platform-package>', 'The package name for the platform in question').choices(
        Object.keys(NAPI_TARGET_MAP),
      ),
    )
    .description('Returns the Rust target triple to use for the given platform package.')
    .action(async (platformPackage) => {
      console.log(NAPI_TARGET_MAP[platformPackage]);
    });

  group
    .command('move-gh-artifacts')
    .argument('<artifacts-path>', 'The path where artifacts were downloaded to')
    .description(
      'Move artifacts downloaded from GitHub during a workflow to their appropriate spots based on their name.',
    )
    .action(async (artifactsPath) => {
      const files = await fs.readdir(artifactsPath);
      for (const basename of files) {
        const rootFilePath = path.resolve(artifactsPath, basename);
        // If the artifact was downloaded as a folder and that folder contains a single file with
        // the same name, then assume that it's meant to be an individual file and use that inner
        // path as the file path in all the logic below.
        const stat = await fs.stat(rootFilePath);
        const isSingleFileArtifact =
          stat.isDirectory() &&
          (await (async () => {
            const innerFiles = await fs.readdir(rootFilePath);
            return innerFiles.length === 1 && innerFiles[0] === basename;
          })());
        const filePath = isSingleFileArtifact ? path.join(rootFilePath, basename) : rootFilePath;

        /**
         * Move the artifact with the given name to the given target. If the artifact is actually a
         * folder with a single file of the same name inside of it, move only that inner file.
         *
         * @param {string} targetPath
         * @returns {Promise<void>}
         */
        async function moveTo(targetPath) {
          console.info('Moving', filePath, 'to', targetPath);
          await fs.rename(filePath, targetPath);
        }

        // Native node extension artifacts move to their platform specific directory.
        const dbExtensionMatch = basename.match(/^intl-message-database\.(.*)\.node$/);
        if (dbExtensionMatch != null) {
          const platform = dbExtensionMatch[1];
          const platformPackage = await pnpm.getPackage(`${NPM_PACKAGES.DATABASE}-${platform}`);
          await moveTo(path.join(platformPackage.path, basename));
          continue;
        }
        // Native node extension artifacts move to their platform specific directory.
        const jsonExtensonMatch = basename.match(/^intl-flat-json-parser\.(.*)\.node$/);
        if (jsonExtensonMatch != null) {
          const platform = jsonExtensonMatch[1];
          const platformPackage = await pnpm.getPackage(`${NPM_PACKAGES.JSON_PARSER}-${platform}`);
          await moveTo(path.join(platformPackage.path, basename));
          continue;
        }

        // SWC transformer .wasm artifact moves into its package folder
        if (/^swc_intl_message_transformer\.wasm$/.test(basename)) {
          const swcPackage = await pnpm.getPackage(NPM_PACKAGES.SWC_TRANSFORMER);
          await moveTo(path.join(swcPackage.path, basename));
          continue;
        }
      }
    });

  return group;
}
