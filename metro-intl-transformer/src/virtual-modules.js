const fs = require('node:fs');
const path = require('node:path');

const VIRTUAL_MODULES_FOLDER = path.resolve(__dirname, '..', '.cache');

function ensureVirtualTranslationsModulesDirectory() {
  fs.mkdirSync(VIRTUAL_MODULES_FOLDER, { recursive: true });
}

/**
 * Returns the absolute path to the virtual translations module that
 * `moduleName` should resolve to.
 * @param {string} moduleName
 * @returns {string}
 */
function getVirtualTranslationsModulePath(moduleName) {
  const resolvedModuleName = moduleName.replace(/[\/\\]+/g, '+').replace(/[:\?<>"\*\|]/g, '!');
  return path.join(VIRTUAL_MODULES_FOLDER, resolvedModuleName);
}

module.exports = {
  VIRTUAL_MODULES_FOLDER,
  ensureVirtualTranslationsModulesDirectory,
  getVirtualTranslationsModulePath,
};
