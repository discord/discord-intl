import { fileURLToPath } from 'node:url';
import path from 'node:path';

export const __filename = fileURLToPath(import.meta.url);
export const __dirname = path.dirname(__filename);

export const REPO_ROOT = path.resolve(__dirname, '..', '..');

export const NPM_PACKAGES = {
  ESLINT_PLUGIN: '@discord/eslint-plugin-discord-intl',
  SWC_TRANSFORMER: '@discord/swc-intl-message-transformer',
  METRO_TRANSFORMER: '@discord/metro-intl-transformer',
  RSPACK_LOADER: '@discord/rspack-intl-loader',
  LOADER_CORE: '@discord/intl-loader-core',
  RUNTIME: '@discord/intl',
  DATABASE: '@discord/intl-message-database',
  JSON_PARSER: '@discord/intl-flat-json-parser',
};

export const CRATES = {
  INTL_MARKDOWN: path.join(REPO_ROOT, 'crates', 'intl_markdown'),
};
