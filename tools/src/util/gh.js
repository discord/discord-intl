/**
 * Utility functions for working around the `gh` cli.
 */
import path from 'node:path';
import { $ } from 'zx';

/**
 * @typedef {{
 *   name: string,
 *   number: number,
 *   status: string,
 *   url: string,
 *   workflowName: string
 * }} WorkflowRun
 */

/**
 * Given the `filePath` of a workflow, return its ID.
 * @param {string} filePath
 * @returns {Promise<string>}
 */
async function getWorkflowIdFromPath(filePath) {
  const output =
    await $`gh workflow list --json id,path,name --jq 'map(select(.path == "${filePath}") | .id)[0]'`;
  return output.stdout.trim();
}

/**
 * Use the `gh` CLI to trigger a new workflow run of the workflow at the given path, using the
 * given arguments supplied as JSON fields for the dispatch event.
 *
 * @param {string} yamlPath
 * @param {string} ref
 * @param {Record<string, string>} args
 */
async function triggerWorkflow(yamlPath, ref, args) {
  return await $`echo ${JSON.stringify(args)} | gh workflow run ${yamlPath} --json --ref ${ref}`;
}

/**
 * Fetches information about the latest run of the given workflow.
 *
 * @param {string} workflowId
 * @returns {Promise<WorkflowRun>}
 */
async function getLatestWorkflowRun(workflowId) {
  const result =
    await $`gh run list -w ${workflowId} --json name,number,workflowName,workflowDatabaseId,url,status --jq ".[0]"`;
  return JSON.parse(result.stdout);
}

/**
 * Wait patiently for GitHub to register that a run has been requested from a
 * workflow_dispatch event. The waiting continues until the latest run returned
 * from the API is greater than `previousRunNumber`.
 *
 * @param {string} workflowId
 * @param {number} previousRunNumber
 * @returns {Promise<WorkflowRun | undefined>}
 */
async function waitForNextRunResponse(workflowId, previousRunNumber) {
  const maxBackoffMs = 10000;
  const maxAttempts = 12;
  let backoffMs = 1000;
  let attempts = 0;
  while (attempts < maxAttempts) {
    attempts += 1;
    const latest = await getLatestWorkflowRun(workflowId);
    if (latest.number > previousRunNumber) return latest;

    await new Promise((resolve) => setTimeout(resolve, backoffMs));
    backoffMs = Math.min(maxBackoffMs, backoffMs * 1.5);
  }

  return undefined;
}

/**
 * Fetch, trigger, and wait for a new run of the workflow defined at `filePath` in this repository
 * to run on GitHub Actions.
 *
 * @param {string} filePath
 * @param {string} ref
 * @param {Record<string, string>} args
 * @returns {Promise<WorkflowRun | undefined>}
 */
async function runWorkflow(filePath, ref, args) {
  const fullPath = path.join('.', '.github', 'workflows', filePath);
  console.log('Gathering workflow information from GitHub...');
  const workflowId = await getWorkflowIdFromPath(fullPath);
  const previousRun = await getLatestWorkflowRun(workflowId);

  console.log(`Triggering workflow ${path.basename(filePath)} on ref ${ref}, with args:`, args);
  await triggerWorkflow(path.basename(filePath), ref, args);
  console.log('Waiting for run request to be registered...');
  const latestRun = await gh.waitForNextRunResponse(workflowId, previousRun.number);
  if (latestRun == null) {
    console.error("Couldn't get a response from GitHub about the latest run. Check manually");
  }

  return latestRun;
}

export const gh = {
  getWorkflowIdFromPath,
  triggerWorkflow,
  getLatestWorkflowRun,
  waitForNextRunResponse,
  runWorkflow,
};
