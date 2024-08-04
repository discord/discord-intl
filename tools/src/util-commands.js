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
        const filePath = path.resolve(artifactsPath, basename);
        // Native node extension artifacts move to their platform specific directory.
        const nativeExtensionMatch = basename.match(/^intl-message-database\.(.*)\.node$/);
        if (nativeExtensionMatch != null) {
          const platform = nativeExtensionMatch[1];
          // If the artifact was downloaded into a subfolder, then the actual artifact is inside of
          // it and needs to be moved out.
          const stats = await fs.stat(filePath);
          const sourcePath = stats.isDirectory() ? path.join(filePath, basename) : filePath;
          const targetPath = path.join('intl_message_database', 'npm', platform, basename);
          console.info('Moving', sourcePath, 'to', targetPath);
          await fs.rename(sourcePath, targetPath);
          continue;
        }
      }
    });

  return group;
}
