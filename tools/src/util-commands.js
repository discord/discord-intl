import fs from 'node:fs/promises';
import path from 'node:path';

import { Argument, Command } from 'commander';

import { NAPI_TARGET_MAP } from './napi.js';

export default async function () {
  const group = new Command('util').description(
    'Random utilities for inspecting or querying things',
  );

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
        const nativeExtensionMatch = basename.match(/^intl-message-database\.(.*)\.node$/);
        if (nativeExtensionMatch != null) {
          const platform = nativeExtensionMatch[1];
          await moveTo(path.join('intl_message_database', 'npm', platform, basename));
          continue;
        }

        // SWC transformer .wasm artifact moves into its package folder
        if (/^swc_intl_message_transformer\.wasm$/.test(basename)) {
          await moveTo(path.join('swc-intl-message-transformer', basename));
          continue;
        }
      }
    });

  return group;
}
