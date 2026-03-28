import { cp, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const extensionRoot = path.resolve(__dirname, '..');
const outVendorRoot = path.join(extensionRoot, 'out', 'vendor', 'etch-wasm');
const wasmSourceRoot = path.resolve(extensionRoot, '..', '..', 'crates', 'etch-wasm', 'pkg');
const testFixtureSourceRoot = path.join(extensionRoot, 'src', 'test', 'fixtures');
const outTestFixtureRoot = path.join(extensionRoot, 'out', 'test', 'fixtures');

await mkdir(outVendorRoot, { recursive: true });
await cp(
  path.join(wasmSourceRoot, 'etch_wasm_bg.wasm'),
  path.join(outVendorRoot, 'etch_wasm_bg.wasm')
);
await mkdir(outTestFixtureRoot, { recursive: true });
await cp(testFixtureSourceRoot, outTestFixtureRoot, { recursive: true });
