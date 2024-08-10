/**
 * Commands for interacting with Github Actions CI directly from the command line.
 * Useful for quickly iterating changes and running pre-configured inputs for
 * workflow sets like "release current commit as canary".
 *
 * These commands rely on the `gh` CLI being installed and authenticated already.
 */
import { Argument, Command } from 'commander';
import { checkbox } from '@inquirer/prompts';

import { gh } from '../util/gh.js';
import { git } from '../util/git.js';
import { pnpm } from '../pnpm.js';

export default async function () {
  const group = new Command('ci').description(
    'Automate running pre-configured workflows on CI (Github Actions)',
  );

  const publicPackages = await pnpm.getPublicPackages();
  const publicPackageNames = publicPackages.map((pack) => pack.name);

  group
    .command('publish-canary')
    .description('Run the "Publish Canary" workflow with the current commit.')
    .addArgument(
      new Argument(
        '[packs...]',
        'Which packages should be published. Will be prompted if not specified',
      ).choices(publicPackageNames),
    )
    .option('--loose', 'Disable strict mode')
    .action(async (packs, { loose }) => {
      if (await git.hasChanges()) {
        console.log(
          'There are uncommited changes. Commit and push them before running this command',
        );
        process.exit(1);
      }

      const chosenPackages =
        packs.length > 0
          ? packs
          : await checkbox({
              message: 'Which packages should be published?',
              choices: publicPackageNames.map((name) => ({ value: name, label: name })),
              required: true,
            });

      console.log('Fetching workflow information from GitHub...');
      const workflowId = await gh.getWorkflowIdFromPath('.github/workflows/publish-canary.yaml');
      const previousRun = await gh.getLatestWorkflowRun(workflowId);
      const ref = git.currentHead();
      console.log('Triggering workflow...');
      await gh.runWorkflow('publish-canary.yaml', {
        commit: ref,
        strict: loose ? 'false' : 'true',
        packages: chosenPackages.join(' '),
      });
      console.log('Waiting for run request to be registered...');
      const latestRun = await gh.waitForNextRunResponse(workflowId, previousRun.number);
      if (latestRun == null) {
        console.error("Couldn't get a response from GitHub about the latest run. Check manually");
        process.exit(1);
      }

      console.log(
        `Created #${latestRun.number} (${latestRun.status}). View the run here: ${latestRun.url}`,
      );
    });

  return group;
}
