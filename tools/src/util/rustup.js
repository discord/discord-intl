import { $ } from 'zx';

/**
 * Check that the given rustup toolchain `target` is installed on the host system.
 *
 * @param {string} targetTriple
 * @returns {Promise<boolean>}
 */
async function hasTargetInstalled(targetTriple) {
  return (await $`rustup target list --installed`).stdout.split('\n').includes(targetTriple);
}

/**
 * Install the given `targetTriple` for the current rustup toolchain. If the target fails to install
 * the returned Promise will be rejected.
 *
 * @param {string} targetTriple
 * @returns {Promise<void>}
 */
async function installTarget(targetTriple) {
  const result = await $`rustup target add ${targetTriple}`;
  if (result.exitCode !== 0) return Promise.reject();
}

/**
 * Ensure that everything needed to build WASM targets is setup on the host system.
 * @returns {Promise<void>}
 */
async function ensureWasmSetup() {
  console.log('Ensuring wasm environment setup for building');
  const hasTargets =
    (await rustup.hasTargetInstalled('wasm32-wasip1')) &&
    (await rustup.hasTargetInstalled('wasm32-unknown-unknown'));
  if (hasTargets) return;
  console.log('Installing wasm32 rust targets');

  await installTarget('wasm32-unknown-unknown');
  await installTarget('wasm32-wasip1');
  console.log('wasm build environment successfully installed');
}

export const rustup = {
  hasTargetInstalled,
  ensureWasmSetup,
};
