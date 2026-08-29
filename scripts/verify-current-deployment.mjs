import { resolveCheckedOutSourceCommit } from './source-commit.mjs';

// A manifest claim follows the exact candidate commit under test, including
// its handoff. Do not accept an inherited EXPECTED_BUILD_SHA: that can
// accidentally verify an older deployment after the candidate changes.
process.env.EXPECTED_BUILD_SHA = resolveCheckedOutSourceCommit();
await import('./verify-production-deployment.mjs');
