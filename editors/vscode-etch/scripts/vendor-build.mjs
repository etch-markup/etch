import { cp, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build, context } from 'esbuild';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export const extensionRoot = path.resolve(__dirname, '..');
export const wasmSourceFile = path.resolve(
  extensionRoot,
  '..',
  '..',
  'crates',
  'etch-wasm',
  'pkg',
  'etch_wasm_bg.wasm'
);

const vendoredEtchKitEntry = path.join(
  extensionRoot,
  'vendor',
  'etch-kit',
  'index.ts'
);
const vendoredEtchKitOutfile = path.join(
  extensionRoot,
  'out',
  'vendor',
  'etch-kit',
  'index.js'
);
const pipelineEntry = path.resolve(
  extensionRoot,
  '..',
  '..',
  'packages',
  'etch-plugin-pipeline',
  'dist',
  'index.js'
);
const vendoredPipelineOutfile = path.join(
  extensionRoot,
  'out',
  'vendor',
  'etch-plugin-pipeline',
  'index.js'
);
const vendoredWasmOutfile = path.join(
  extensionRoot,
  'out',
  'vendor',
  'etch-wasm',
  'etch_wasm_bg.wasm'
);
const testFixtureSourceRoot = path.join(extensionRoot, 'src', 'test', 'fixtures');
const outTestFixtureRoot = path.join(extensionRoot, 'out', 'test', 'fixtures');

export async function copyWasmAsset() {
  await mkdir(path.dirname(vendoredWasmOutfile), { recursive: true });
  await cp(wasmSourceFile, vendoredWasmOutfile);
}

export async function copyTestFixtures() {
  await mkdir(outTestFixtureRoot, { recursive: true });
  await cp(testFixtureSourceRoot, outTestFixtureRoot, { recursive: true });
}

export async function buildVendoredEtchKit() {
  await mkdir(path.dirname(vendoredEtchKitOutfile), { recursive: true });
  await build(createBuildOptions(vendoredEtchKitEntry, vendoredEtchKitOutfile));
}

export async function buildVendoredPipeline() {
  await mkdir(path.dirname(vendoredPipelineOutfile), { recursive: true });
  await build(createBuildOptions(pipelineEntry, vendoredPipelineOutfile));
}

export async function createVendoredEtchKitContext() {
  await mkdir(path.dirname(vendoredEtchKitOutfile), { recursive: true });
  return context(createBuildOptions(vendoredEtchKitEntry, vendoredEtchKitOutfile));
}

export async function createVendoredPipelineContext() {
  await mkdir(path.dirname(vendoredPipelineOutfile), { recursive: true });
  return context(createBuildOptions(pipelineEntry, vendoredPipelineOutfile));
}

function createBuildOptions(entryPoint, outfile) {
  return {
    entryPoints: [entryPoint],
    bundle: true,
    format: 'esm',
    platform: 'node',
    outfile,
    minify: true,
    legalComments: 'none',
    sourcemap: false,
    target: 'node20',
    external: ['node:*'],
  };
}
