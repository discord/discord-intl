import fs from 'node:fs/promises';
import path from 'node:path';

import { NapiCli, parseTriple } from '@napi-rs/cli';

const napiCli = new NapiCli();

export const NAPI_TARGET_MAP = {
  'darwin-arm64': 'aarch64-apple-darwin',
  'darwin-x64': 'x86_64-apple-darwin',
  'linux-arm64-gnu': 'aarch64-unknown-linux-gnu',
  'linux-arm64-musl': 'aarch64-unknown-linux-musl',
  'linux-x64-gnu': 'x86_64-unknown-linux-gnu',
  'linux-x64-musl': 'x86_64-unknown-linux-musl',
  'win32-arm64-msvc': 'aarch64-pc-windows-msvc',
  'win32-ia32-msvc': 'i686-pc-windows-msvc',
  'win32-x64-msvc': 'x86_64-pc-windows-msvc',
  wasm: 'wasm32-wasip1',
  'android-arm-eabi': 'invalid',
  'android-arm64': 'invalid',
  'freebsd-x64': 'invalid',
  'linux-arm-gnueabihf': 'invalid',
  'linux-arm-musleabihf': 'invalid',
  'linux-riscv64-gnu': 'invalid',
  'linux-riscv64-musl': 'invalid',
  'linux-s390x-gnu': 'invalid',
};

/**
 * Build a NAPI-RS platform package
 * @param {string} baseName Root of the .node file built by cargo, e.g. `intl-message-database`.
 * @param {import('./pnpm.js').PnpmPackage} pack
 * @param {keyof typeof NAPI_TARGET_MAP} platformPackage
 */
export async function buildNapiPackage(baseName, pack, platformPackage) {
  const targetTriple = NAPI_TARGET_MAP[platformPackage];

  const hostPlatform = process.platform;
  const hostArch = process.arch;
  const target = parseTriple(targetTriple);

  // Windows has a new target version? Something? Setting this to a static 16 is the only way it
  // successfully builds right now.
  if (target.platform === 'win32') {
    process.env['XWIN_VERSION'] = '16';
  }

  const buildResult = await napiCli.build({
    cwd: pack.path,
    target: targetTriple,
    crossCompile:
      hostPlatform !== target.platform ||
      hostArch !== target.arch ||
      // We're always gonna be building on a gnu host for linux, so if the target abi is different,
      // then we'll need to cross-compile.
      (target.platform === 'linux' && target.abi !== 'gnu'),
    platform: true,
    profile: 'release',
    // We've re-written the js binding to be a lot smaller and not have implicit TS errors.
    noJsBinding: true,
  });

  // The buildResult is just a container, the task is the actual thing doing the building, so we
  // need to wait for that to finish, too.
  await buildResult.task;

  // NAPI automatically copies the build artifact with an appropriate name into the crate's root
  // folder, so we just need to move it over into the npm package.
  const artifactName = `${baseName}.${platformPackage}.node`;
  await fs.rename(
    path.resolve(pack.path, artifactName),
    path.resolve(pack.path, `npm`, platformPackage, artifactName),
  );
}
