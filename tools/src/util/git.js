import { $ } from 'zx';

/**
 * Return the full SHA of the current commit. If `short` is specified
 * @param {{
 *   short?: boolean,
 * }} options
 * @returns {string}
 */
function currentHead(options = {}) {
  const { short = false } = options;
  return $.sync`git rev-parse ${short ? '--short' : ''} HEAD`.stdout.trim();
}

/**
 * Returns true if the current git state contains any changes.
 *
 * @returns {Promise<boolean>}
 */
async function hasChanges() {
  const status = await $`git status --porcelain`;
  return status.stdout.trim().length > 0;
}

/**
 * If the current git state has any changes, log an error message and reject execution. The caller
 * can either allow the Promise rejection to propagate, or handle it separately. If `hardExit` is
 * true, the process will be aborted directly after logging the message.
 *
 * @param {boolean=} hardExit
 * @returns {Promise<void>}
 */
async function rejectIfHasChanges(hardExit = false) {
  if (!(await hasChanges())) return;

  const errorMessage =
    'There are uncommited changes. Commit and push them before running this command';

  if (hardExit) {
    console.log(errorMessage);
    process.exit(1);
  }

  return Promise.reject(errorMessage);
}

export const git = {
  currentHead,
  hasChanges,
  rejectIfHasChanges,
};
