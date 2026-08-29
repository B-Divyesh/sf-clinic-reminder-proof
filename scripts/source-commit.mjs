import { spawnSync } from 'node:child_process';
import { buildShaFromImage, normalizeBuildSha } from './deployment-identity.mjs';

// The release identity covers both bytes that enter the container and the
// checked-in control-plane template that makes those bytes safe to run. Files
// under .factory are deliberately absent: handoff and verifier evidence do
// not change a release. The deployment scripts are deliberately present: a
// change to how a revision is composed or verified must be built, tagged, and
// deployed as a new release candidate.
export const RELEASE_BUILD_INPUTS = [
  'Dockerfile',
  '.dockerignore',
  'package.json',
  'package-lock.json',
  'tsconfig.json',
  'vitest.config.ts',
  'playwright.config.ts',
  'Cargo.toml',
  'Cargo.lock',
  'apps/web',
  'packages/design-system',
  'services/api',
  'deployment/containerapp.json',
  'scripts/containerapp-topology.mjs',
  'scripts/deploy-containerapp.mjs',
  'scripts/deployment-identity.mjs',
  'scripts/source-commit.mjs',
  'scripts/verify-current-deployment.mjs',
  'scripts/verify-production-deployment.mjs'
];

/**
 * Resolve the newest release-affecting revision. The factory records
 * verification evidence in .factory after a release; that directory is
 * intentionally excluded so documentation-only commits do not turn the
 * immutable runtime identity check into a false deployment failure.
 */
export function resolveCheckedOutSourceCommit(run = spawnSync) {
  const result = run('git', ['log', '-1', '--format=%H', '--', ...RELEASE_BUILD_INPUTS], { encoding: 'utf8' });
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || 'git log failed';
    throw new Error(`cannot resolve the checked-out release source commit: ${detail}`);
  }
  return normalizeBuildSha(result.stdout.trim(), 'checked-out release source commit');
}

/**
 * A full immutable tag alone is not enough. Deploying a previous full tag can
 * still leave the current candidate unserved, so bind the requested image to
 * the release revision that the production claim will later verify.
 */
export function assertDeploymentImageMatchesSource(image, sourceCommit) {
  const imageCommit = buildShaFromImage(image);
  const expectedCommit = normalizeBuildSha(sourceCommit, 'checked-out release source commit');
  if (imageCommit !== expectedCommit) {
    throw new Error(`deployment image build SHA must match the checked-out release source commit; expected ${expectedCommit}, got ${imageCommit}`);
  }
  return imageCommit;
}
