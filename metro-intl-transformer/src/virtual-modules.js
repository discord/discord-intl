const fs = require('node:fs');
const path = require('node:path');

/**
 * Ensure the configured cache directory exists so that files can be read from and written to it.
 *
 * @param {string} cacheDir
 */
function ensureVirtualTranslationsModulesDirectory(cacheDir) {
  if (!fs.existsSync(cacheDir)) {
    fs.mkdirSync(cacheDir, { recursive: true });
  }
}

/**
 * Returns the absolute path to the virtual translations module that
 * `moduleName` should resolve to.
 * @param {string} cacheDir
 * @param {string} moduleName
 * @returns {string}
 */
function getVirtualTranslationsModulePath(cacheDir, moduleName) {
  const resolvedModuleName = moduleName.replace(/[\/\\]+/g, '+').replace(/[:\?<>"\*\|]/g, '!');
  return path.resolve(cacheDir, resolvedModuleName);
}

module.exports = {
  ensureVirtualTranslationsModulesDirectory,
  getVirtualTranslationsModulePath,
};
