import {
  buildVendoredEtchKit,
  buildVendoredPipeline,
  copyTestFixtures,
  copyWasmAsset,
} from './vendor-build.mjs';

await copyWasmAsset();
await buildVendoredEtchKit();
await buildVendoredPipeline();
await copyTestFixtures();
