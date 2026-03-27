import * as path from 'node:path';
import Mocha = require('mocha');
import { glob } from 'glob';

export async function run(): Promise<void> {
  const mocha = new Mocha({
    ui: 'tdd',
    color: true,
    timeout: 20000,
  });

  const testsRoot = __dirname;
  const files = await glob('**/*.test.cjs', {
    cwd: testsRoot,
    absolute: true,
  });

  for (const file of files) {
    mocha.addFile(path.resolve(file));
  }

  await mocha.loadFilesAsync();

  await new Promise<void>((resolve, reject) => {
    mocha.run((failures) => {
      if (failures > 0) {
        reject(new Error(`${failures} tests failed.`));
        return;
      }

      resolve();
    });
  });
}
