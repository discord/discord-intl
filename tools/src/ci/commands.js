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

/**
 * @param {import('../util/gh.js').WorkflowRun | undefined} run
 */
function logWorkflowRunResponseOrExit(run) {
  if (run == null) {
    console.error("Couldn't get a response from GitHub about the latest run. Check manually");
    process.exit(1);
  }

  console.log(`Created #${run.number} (${run.status}). View the run here: ${run.url}`);
}

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
      await git.rejectIfHasChanges(true);

      const chosenPackages =
        packs.length > 0
          ? packs
          : await checkbox({
              message: 'Which packages should be published?',
              choices: publicPackageNames.map((name) => ({ value: name, label: name })),
              required: true,
            });

      const run = await gh.runWorkflow('publish-canary.yaml', {
        commit: git.currentHead(),
        strict: loose ? 'false' : 'true',
        packages: chosenPackages.join(' '),
      });

      logWorkflowRunResponseOrExit(run);
    });

  group
    .command('release')
    .description('Run the "Release" workflow, publishing a new version of the entire ecosystem.')
    .option('--dry-run', "Run the workflow, but don't actually publish packages.")
    .option(
      '--tag <tag>',
      'Tag to additionally apply to the published package. Defaults to `latest`.',
      'latest',
    )
    .option('--fail-fast', 'Cancel the rest of the build after the first failure')
    .action(async ({ dryRun, tag, failFast }) => {
      await git.rejectIfHasChanges(true);

      console.log({
        commit: git.currentHead(),
        publish: dryRun ? 'false' : 'true',
        'fail-fast': failFast ? 'true' : 'false',
        tag,
      });

      const run = await gh.runWorkflow('release.yaml', {
        commit: git.currentHead(),
        publish: dryRun ? 'false' : 'true',
      });

      logWorkflowRunResponseOrExit(run);
    });

  return group;
}
