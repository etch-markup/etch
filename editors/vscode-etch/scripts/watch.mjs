import { spawn } from 'node:child_process';
import { watch } from 'node:fs';
import process from 'node:process';

import {
  buildWasmPackage,
  copyTestFixtures,
  copyWasmAsset,
  createVendoredEtchKitContext,
  createVendoredPipelineContext,
  extensionRoot,
  wasmManifestFile,
  wasmSourceRoot,
} from './vendor-build.mjs';

const pnpmCommand = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
const childProcesses = [];
const wasmWatchers = [];
let shuttingDown = false;
let wasmBuildRunning = false;
let wasmBuildQueued = false;

await runInitialCompile();

const vendoredEtchKitContext = await createVendoredEtchKitContext();
await vendoredEtchKitContext.watch();

const vendoredPipelineContext = await createVendoredPipelineContext();
await vendoredPipelineContext.watch();

await copyWasmAsset();
await copyTestFixtures();

wasmWatchers.push(
  watch(wasmManifestFile, () => {
    void rebuildWasmArtifacts();
  }),
  watch(wasmSourceRoot, () => {
    void rebuildWasmArtifacts();
  })
);

childProcesses.push(
  spawnWatcher('etch-kit', [
    '--dir',
    '../..',
    '--filter',
    '@etch-markup/etch-kit',
    'watch',
  ]),
  spawnWatcher('etch-plugin-sdk', [
    '--dir',
    '../..',
    '--filter',
    '@etch-markup/etch-plugin-sdk',
    'watch',
  ]),
  spawnWatcher('etch-plugin-pipeline', [
    '--dir',
    '../..',
    '--filter',
    '@etch-markup/etch-plugin-pipeline',
    'watch',
  ]),
  spawnWatcher('extension-tsc', [
    'exec',
    'tsc',
    '-w',
    '-p',
    './',
    '--preserveWatchOutput',
  ])
);

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    void shutdown(0);
  });
}

await new Promise(() => undefined);

function spawnWatcher(label, args) {
  const child = spawn(pnpmCommand, args, {
    cwd: extensionRoot,
    stdio: 'inherit',
    env: process.env,
  });

  child.on('exit', (code, signal) => {
    if (shuttingDown) {
      return;
    }

    if (signal) {
      console.error(`[watch:${label}] exited from signal ${signal}`);
    } else if (code !== 0) {
      console.error(`[watch:${label}] exited with code ${code ?? 'unknown'}`);
    }

    void shutdown(code ?? 1);
  });

  return child;
}

async function runInitialCompile() {
  await new Promise((resolve, reject) => {
    const child = spawn(pnpmCommand, ['run', 'compile'], {
      cwd: extensionRoot,
      stdio: 'inherit',
      env: process.env,
    });

    child.on('exit', (code) => {
      if (code === 0) {
        resolve(undefined);
        return;
      }

      reject(new Error(`Initial compile failed with code ${code ?? 'unknown'}.`));
    });
  });
}

async function shutdown(exitCode) {
  if (shuttingDown) {
    return;
  }

  shuttingDown = true;

  for (const watcher of wasmWatchers) {
    watcher.close();
  }

  for (const child of childProcesses) {
    child.kill('SIGTERM');
  }

  await Promise.allSettled([
    vendoredEtchKitContext.dispose(),
    vendoredPipelineContext.dispose(),
  ]);

  process.exit(exitCode);
}

async function rebuildWasmArtifacts() {
  if (wasmBuildRunning) {
    wasmBuildQueued = true;
    return;
  }

  wasmBuildRunning = true;

  try {
    do {
      wasmBuildQueued = false;
      await buildWasmPackage();
      await copyWasmAsset();
    } while (wasmBuildQueued);
  } catch (error) {
    console.error('[watch:wasm] Failed to rebuild the wasm package.');
    console.error(error);
  } finally {
    wasmBuildRunning = false;
  }
}
