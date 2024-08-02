import { $, cd } from 'zx';
import { getPackage } from '../pnpm.js';

export const TARGET_PACKAGES = {
  'darwin-arm64': 'aarch64-apple-darwin',
  'darwin-x64': 'x86_64-apple-darwin',
  'linux-arm64-gnu': 'aarch64-unknown-linux-gnu',
  'linux-arm64-musl': 'aarch64-unknown-linux-musl',
  'linux-x64-gnu': 'x86_64-unknown-linux-gnu',
  'linux-x64-musl': 'x86_64-unknown-linux-musl',
  'win32-arm64-msvc': 'aarch64-pc-windows-msvc',
  'win32-ia32-msvc': 'i686-pc-windows-msvc',
  'win32-x64-msvc': 'x86_64-pc-windows-msvc',
};

/**
 * @typedef {keyof typeof TARGET_PACKAGES} TargetPackage
 */

/**
 * Build the native Node extension for the given target. After building, the `.node` artifact will
 * be copied into the target package's npm folder, making it ready for publishing.
 * @param {TargetPackage} targetPackage
 */
export async function buildNodeExtension(targetPackage) {
  if (targetPackage == null) {
    throw new Error('Target was not specified for building intl_message_database');
  }
  if (TARGET_PACKAGES[targetPackage] == null) {
    throw new Error(`Target ${targetPackage} is not a known package for intl_message_database`);
  }

  const dbPackage = await getPackage('@discord/intl-message-database');

  await $({
    cwd: dbPackage.path,
    env: {
      ...process.env,
      PACKAGE_NAME: targetPackage,
      CARGO_BUILD_TARGET: TARGET_PACKAGES[targetPackage],
    },
    stdio: 'inherit',
  })`pnpm build:artifact`;
}
