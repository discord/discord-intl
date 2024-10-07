import { program } from 'commander';
import { $, cd } from 'zx';

import { REPO_ROOT } from './src/constants.js';
import utilCommands from './src/util-commands.js';
import ciCommands from './src/ci/commands.js';
import dbCommands from './src/db/commands.js';
import ecosystemCommands from './src/ecosystem/commands.js';
import { createJsPackageCommands } from './src/js-package.js';
import { rustup } from './src/util/rustup.js';

process.chdir(REPO_ROOT);
cd(REPO_ROOT);

(async () => {
  program
    .description('Internal tooling for managing the discord-intl repo and packages.')
    .addCommand(await ciCommands())
    .addCommand(await dbCommands())
    .addCommand(await ecosystemCommands())
    .addCommand(await utilCommands())
    .addCommand(
      await createJsPackageCommands('eslint-plugin-discord-intl', { aliases: ['eslint'] }),
    )
    .addCommand(await createJsPackageCommands('metro-intl-transformer', { aliases: ['metro'] }))
    .addCommand(await createJsPackageCommands('rspack-intl-loader', { aliases: ['rspack'] }))
    .addCommand(
      await createJsPackageCommands('intl', {
        aliases: ['rt', 'runtime'],
        build: true,
        watch: true,
      }),
    )
    .addCommand(await createJsPackageCommands('intl-loader-core', { aliases: ['loader'] }))
    .addCommand(
      await createJsPackageCommands('swc-intl-message-transformer', {
        aliases: ['swc'],
        prebuild: rustup.ensureWasmSetup,
        build: true,
      }),
    )
    .parse();
})();
