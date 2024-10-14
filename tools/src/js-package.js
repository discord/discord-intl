import chokidar from 'chokidar';
import { Command } from 'commander';
import { $ } from 'zx';

import { npmPublishCommand } from './npm.js';
import { getPackage } from './pnpm.js';
import { versionCommand } from './versioning.js';

const DEFAULT_IGNORE_PATTERNS = ['**/node_modules/**', '**/dist/**'];

/**
 * Run the `build` script defined in `pack`'s package.json once.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @returns {Promise<void>}
 */
async function runBuildScript(pack) {
  await $({ cwd: pack.path, stdio: 'inherit' })`pnpm build`;
}

/**
 * Watch the given `watchPaths` for changes and re-run the build script every time a change is
 * observed.
 *
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {string[]} watchPatterns
 * @returns {Promise<void>}
 */
async function watchBuild(pack, watchPatterns) {
  console.log(`Setting up file watching for ${pack.name}:`);
  for (const pattern of watchPatterns) {
    console.log(`+ ${pattern}`);
  }

  async function buildAndCatch() {
    await runBuildScript(pack)
      .then(() => {
        console.warn('Build succeeded');
      })
      .catch((error) => {
        console.error(`Build failed: `, error);
      });
  }
  chokidar
    .watch(watchPatterns, {
      ignored: DEFAULT_IGNORE_PATTERNS,
      ignoreInitial: true,
      persistent: true,
    })
    .on('all', buildAndCatch)
    .on('ready', buildAndCatch);
}

/**
 * Generate a base set of commands for managing a JavaScript package, including versioning and
 * publishing to npm.
 *
 * @param {string} name
 * @param {{
 *   aliases?: string[],
 *   build?: boolean,
 *   prebuild?: () => Promise<void>,
 *   watch?: boolean | string[]
 * }=} options
 * @returns {Promise<Command>}
 */
export async function createJsPackageCommands(name, options = {}) {
  const { build = false, prebuild, watch, aliases = [] } = options;
  const pack = await getPackage('@discord/' + name);

  const group = new Command(name).aliases(aliases).description(`Operate on the ${name} package`);
  group.addCommand(npmPublishCommand('publish', pack));
  group.addCommand(versionCommand('version', pack));

  if (build) {
    group
      .command('build')
      .description(`Build the ${name} package`)
      .action(async () => {
        if (prebuild != null) await prebuild();
        await $({ cwd: pack.path, stdio: 'inherit' })`pnpm build`;
      });
  }

  if (watch) {
    group
      .command('watch')
      .description(
        `Run the build process for ${name} and automatically reload changes when watched files change.`,
      )
      .action(async () => {
        const paths = Array.isArray(watch)
          ? watch.map((pattern) => pack.path + '/' + pattern)
          : [pack.path + '/**'];
        await watchBuild(pack, paths);
      });
  }

  return group;
}
