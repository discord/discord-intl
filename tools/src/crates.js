import { $ } from 'zx';
import { Command } from 'commander';

/**
 * Run `pnpm publish` with the given arguments.
 */
export async function cratesPublish() {
  await $({ stdio: 'inherit' })`cargo workspaces publish --publish-as-is`;
  console.info('Finished publishing crates');
}

export async function cratesCommands() {
  const group = new Command('crates').description(
    'Commands for interacting with Rust crates in the workspace',
  );

  group
    .command('publish-all')
    .description('Publish all public crates in the workspace')
    .action(async () => {
      await cratesPublish();
    });

  return group;
}
