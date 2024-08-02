import { program } from 'commander';
import { cd } from 'zx';

import { REPO_ROOT } from './src/constants.js';
import dbCommands from './src/db/commands.js';
import swcCommands from './src/swc/commands.js';

process.chdir(REPO_ROOT);
cd(REPO_ROOT);

(async () => {
  program
    .description('Internal tooling for managing the discord-intl repo and packages.')
    .addCommand(await dbCommands())
    .addCommand(await swcCommands())
    .parse();
})();
