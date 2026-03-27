import { defineConfig } from '@vscode/test-cli';

export default defineConfig({
  files: 'out/test/suite/**/*.test.cjs',
  version: 'insiders',
  mocha: {
    ui: 'tdd',
    color: true,
    timeout: 20000,
  },
});
