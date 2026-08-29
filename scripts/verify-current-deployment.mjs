import { resolveCheckedOutSourceCommit } from './source-commit.mjs';

// A manifest claim must follow the source revision under test. Do not accept
// an inherited EXPECTED_BUILD_SHA: that can accidentally verify an older
// deployment after a source change.
process.env.EXPECTED_BUILD_SHA = resolveCheckedOutSourceCommit();
await import('./verify-production-deployment.mjs');
