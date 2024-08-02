import { fileURLToPath } from 'node:url';
import path from 'node:path';

export const __filename = fileURLToPath(import.meta.url);
export const __dirname = path.dirname(__filename);

export const REPO_ROOT = path.resolve(__dirname, '..');
