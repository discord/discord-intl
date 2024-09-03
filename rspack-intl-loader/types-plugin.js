const {
  database,
  isMessageDefinitionsFile,
  generateTypeDefinitions,
} = require('@discord/intl-loader-core');

/**
 * A plugin that watches for changes to I18n strings and updates messages.d.ts (and its sourcemap) automatically.
 */
class IntlTypeGeneratorPlugin {
  /**
   * @param {{
   *   allowNullability?: boolean
   * }} options
   */
  constructor(options = {}) {
    this.allowNullability = options.allowNullability ?? false;
  }
  /**
   * @param {string} filePath
   * @returns {number} How long it took to generate the type definitions file.
   */
  generateTypeDefinitions(filePath) {
    const start = performance.now();
    generateTypeDefinitions(filePath, undefined, this.allowNullability);
    const end = performance.now();

    return end - start;
  }

  generateAllTypes() {
    const paths = database.getAllSourceFilePaths();
    let totalDuration = 0;

    for (const path of paths) {
      if (isMessageDefinitionsFile(path)) {
        totalDuration += this.generateTypeDefinitions(path);
      }
    }

    console.error(
      `🌍 Updated all intl type definitions (${paths.length} files, ${totalDuration.toFixed(3)}ms)`,
    );
  }

  /** @param {import('webpack').Compiler} compiler */
  apply(compiler) {
    let isFirstCompilation = true;
    compiler.hooks.afterCompile.tap('IntlTypeGeneratorPlugin', () => {
      if (isFirstCompilation) {
        this.generateAllTypes();
        isFirstCompilation = false;
      }
    });
    compiler.hooks.invalid.tap('IntlTypeGeneratorPlugin', (filePath) => {
      if (filePath != null && isMessageDefinitionsFile(filePath)) {
        const duration = this.generateTypeDefinitions(filePath);
        console.error(
          `🌍 Updated intl type definitions for ${filePath} (${duration.toFixed(3)}ms)`,
        );
      }
    });
  }
}

module.exports = { IntlTypeGeneratorPlugin };
