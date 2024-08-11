import { IntlSourceFile } from '@discord/intl-message-database';

export interface MessageDefinitionsTransformerOptions {
  /**
   * The list of hashed message keys that this file manages.
   */
  messageKeys: string[];
  /**
   * Map of locale names to import paths used for loading translations.
   */
  localeMap: Record<string, string>;
  /**
   * Function to create a prelude that gets injected at the start of the transformed file to set up
   * anything needed for other injections later on.
   */
  getPrelude?: () => string;
  /**
   * Function to generate an import/require statement for the compiled asset file. All imports
   * should be asynchronous (e.g., typically use `import` rather than `require`), but some platforms
   * implement loading differently and may need different syntax. For example, React Native Assets
   * are bundled using `require` statements to return an Asset ID, which can then be loaded
   * asynchronously by some other code to get the actual content of the asset.
   *
   * The code created by this function must create a `Promise<{default: Record<string, any>}>`. In
   * other words, a Promise for an object with a `default` key pointing to an object map of message
   * keys to their values. For `import` statements, this is already the default. For `requires`, you
   * may need to wrap the result with the `default` key, like `.then((data) => ({default: data}))`.
   */
  getTranslationImport(importPath: string): string;
}

/**
 * The result of calling `processDefinitionsFile`, including the created source file, locale map,
 * and more.
 */
export interface ProcessDefinitionsResult {
  /**
   * Direct source file from the database that was created or updated by this process.
   */
  sourceFile: IntlSourceFile;
  /**
   * The full list of message keys contained by the processed source file. While `sourceFile`
   * contains a list of key _symbols_, this list contains all of the resolved strings for the
   * hashed message keys.
   */
  hashedMessageKeys: string[];
  /**
   * Fully-resolved path to the translations directory that was scanned for entries for the
   * source file.
   */
  translationsPath: string;
  /**
   * Map of locale names to file paths for all translations files that were discovered when scanning
   * the configured `translationsPath`. Note that this _does not_ include the source locale, since
   * it's target is often different between loaders (e.g., could be a virtual file, an asset that
   * gets compiled separately, or use query parameters to control loader behavior when reusing the
   * same file).
   */
  translationsLocaleMap: Record<string, string>;
}

/**
 * The result of calling `processTranslationsFile`, including the created source file, locale map,
 * and more.
 */
export interface ProcessTranslationsResult {
  /**
   * Direct source file from the database that was created or updated by this process.
   */
  sourceFile: IntlSourceFile;
  /**
   * The full list of message keys contained by the processed source file. While `sourceFile`
   * contains a list of key _symbols_, this list contains all of the resolved strings for the
   * hashed message keys.
   */
  hashedMessageKeys: string[];
}
