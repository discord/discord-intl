import { program } from 'commander';
import { cd } from 'zx';

import { REPO_ROOT } from './src/constants.js';
import utilCommands from './src/util-commands.js';
import dbCommands from './src/db/commands.js';
import ecosystemCommands from './src/ecosystem/commands.js';
import metroCommands from './src/metro/commands.js';
import rspackCommands from './src/rspack/commands.js';
import runtimeCommands from './src/runtime/commands.js';
import swcCommands from './src/swc/commands.js';

process.chdir(REPO_ROOT);
cd(REPO_ROOT);

(async () => {
  program
    .description('Internal tooling for managing the discord-intl repo and packages.')
    .addCommand(await dbCommands())
    .addCommand(await ecosystemCommands())
    .addCommand(await utilCommands())
    .addCommand(await metroCommands())
    .addCommand(await rspackCommands())
    .addCommand(await runtimeCommands())
    .addCommand(await swcCommands())
    .parse();
})();
