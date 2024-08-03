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

  return group;
}
