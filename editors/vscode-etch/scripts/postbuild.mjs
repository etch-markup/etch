import { cp, mkdir } from 'node:fs/promises';
import { build } from 'esbuild';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const extensionRoot = path.resolve(__dirname, '..');
const outVendorRoot = path.join(extensionRoot, 'out', 'vendor', 'etch-wasm');
const wasmSourceRoot = path.resolve(extensionRoot, '..', '..', 'crates', 'etch-wasm', 'pkg');
const etchKitEntry = path.join(extensionRoot, 'src', 'vendor', 'etch-kit', 'index.ts');
const outEtchKitRoot = path.join(extensionRoot, 'out', 'vendor', 'etch-kit');
const pipelineEntry = path.resolve(
  extensionRoot,
  '..',
  '..',
  'packages',
  'etch-plugin-pipeline',
  'dist',
  'index.js'
);
const outPipelineRoot = path.join(extensionRoot, 'out', 'vendor', 'etch-plugin-pipeline');
const testFixtureSourceRoot = path.join(extensionRoot, 'src', 'test', 'fixtures');
const outTestFixtureRoot = path.join(extensionRoot, 'out', 'test', 'fixtures');

await mkdir(outVendorRoot, { recursive: true });
await cp(
  path.join(wasmSourceRoot, 'etch_wasm_bg.wasm'),
  path.join(outVendorRoot, 'etch_wasm_bg.wasm')
);
await mkdir(outEtchKitRoot, { recursive: true });
await build({
  entryPoints: [etchKitEntry],
  bundle: true,
  format: 'esm',
  platform: 'node',
  outfile: path.join(outEtchKitRoot, 'index.js'),
  minify: true,
  legalComments: 'none',
  sourcemap: false,
  target: 'node20',
  external: ['node:*'],
});
await mkdir(outPipelineRoot, { recursive: true });
await build({
  entryPoints: [pipelineEntry],
  bundle: true,
  format: 'esm',
  platform: 'node',
  outfile: path.join(outPipelineRoot, 'index.js'),
  minify: true,
  legalComments: 'none',
  sourcemap: false,
  target: 'node20',
  external: ['node:*'],
});
await mkdir(outTestFixtureRoot, { recursive: true });
await cp(testFixtureSourceRoot, outTestFixtureRoot, { recursive: true });
