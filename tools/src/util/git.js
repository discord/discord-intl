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

async function hasChanges() {
  const status = await $`git status --porcelain`;
  return status.stdout.trim().length > 0;
}

export const git = {
  currentHead,
  hasChanges,
};
