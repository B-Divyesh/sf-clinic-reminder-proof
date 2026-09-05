import { readFile } from 'node:fs/promises';
import { normalizeBuildSha } from './deployment-identity.mjs';
import { resolveCheckedOutSourceCommit } from './source-commit.mjs';

async function runtimeImplementationSha() {
  if (process.env.REMINDER_PROOF_RUNTIME_SHA) {
    return normalizeBuildSha(process.env.REMINDER_PROOF_RUNTIME_SHA, 'REMINDER_PROOF_RUNTIME_SHA');
  }

  try {
    const release = JSON.parse(await readFile(new URL('../.factory/runtime-release.json', import.meta.url), 'utf8'));
    return normalizeBuildSha(release.implementation_sha, '.factory/runtime-release.json implementation_sha');
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
    return resolveCheckedOutSourceCommit();
  }
}

// Documentation-only evidence commits do not require a new product image.
// The checked-in release record binds the live check to the last implementation
// commit; an explicit environment value is reserved for one-off investigations.
process.env.EXPECTED_BUILD_SHA = await runtimeImplementationSha();
await import('./verify-production-deployment.mjs');
