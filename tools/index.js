import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { program } from 'commander';
import { cd } from 'zx';

import dbCommands from './src/db/commands.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '..');

process.chdir(REPO_ROOT);
cd(REPO_ROOT);

program
  .description('Internal tooling for managing the discord-intl repo and packages.')
  .addCommand(dbCommands())
  .parse();
