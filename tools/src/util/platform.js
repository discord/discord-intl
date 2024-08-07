import fs from 'node:fs';
import path from 'node:path';

import { __dirname } from '../constants.js';
import { NAPI_TARGET_MAP } from '../napi.js';

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim();
      return fs.readFileSync(lddPath, 'utf8').includes('musl');
    } catch (e) {
      return true;
    }
  } else {
    /** @type {any} */
    let report = process.report.getReport();
    if (typeof report === 'string') {
      report = JSON.parse(report);
    }
    const { glibcVersionRuntime } = report.header;
    return !glibcVersionRuntime;
  }
}

/** @type {Record<string, Record<string, string>>} */
const PACKAGE_NAMES = {
  android: { arm: 'android-arm-eabi', arm64: 'android-arm64' },
  win32: { arm64: 'win32-arm64-msvc', ia32: 'win32-ia32-msvc', x64: 'win32-x64-msvc' },
  darwin: { arm64: 'darwin-arm64', x64: 'darwin-x64' },
  freebsd: { x64: 'freebsd-x64' },
  'linux-gnu': {
    arm: 'linux-arm-gnueabihf',
    arm64: 'linux-arm64-gnu',
    x64: 'linux-x64-gnu',
    riscv64: 'linux-riscv64-gnu',
    s390x: 'linux-s390x-gnu',
  },
  'linux-musl': {
    arm: 'linux-arm-musleabihf',
    arm64: 'linux-arm64-musl',
    x64: 'linux-x64-musl',
    riscv64: 'linux-riscv64-musl',
  },
};

const platform =
  process.platform !== 'linux' ? process.platform : 'linux-' + (isMusl() ? 'musl' : 'gnu');
const arch = process.arch;

/**
 * @returns {string}
 */
function getPackageName() {
  if (!(platform in PACKAGE_NAMES)) {
    throw new Error(`Unsupported OS: ${platform}`);
  }
  if (!(arch in PACKAGE_NAMES[platform])) {
    throw new Error(`Unsupported architecture for ${platform}: ${arch}`);
  }
  return PACKAGE_NAMES[platform][arch];
}

const packageName = getPackageName();
const localPath = path.join(
  __dirname,
  `npm/${packageName}/intl-message-database.${packageName}.node`,
);
const packagePath = `@discord/intl-message-database-${packageName}`;

/**
 * Information about the host system that's running this command, usable for creating default
 * targets and argument values that rely on a target platform or package.
 * @type {{packagePath: string, triple: *, localPath: string, target: string}}
 */
const hostPlatform = {
  target: packageName,
  triple: NAPI_TARGET_MAP[packageName],
  packagePath,
  localPath,
};

export { hostPlatform };
