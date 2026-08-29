import { spawnSync } from 'node:child_process';
import { buildShaFromImage, normalizeBuildSha } from './deployment-identity.mjs';

/**
 * The factory verifies the exact candidate commit, including a handoff-only
 * commit. Bind build and runtime identity to HEAD so a later documentation
 * commit cannot be accepted while an older image remains public.
 */
export function resolveCheckedOutSourceCommit(run = spawnSync) {
  const result = run('git', ['rev-parse', '--verify', 'HEAD'], { encoding: 'utf8' });
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || 'git rev-parse failed';
    throw new Error(`cannot resolve the checked-out candidate commit: ${detail}`);
  }
  return normalizeBuildSha(result.stdout.trim(), 'checked-out candidate commit');
}

/**
 * A release must be immutable and obtainable by the independent verifier.
 * Refuse a rollout when tracked files differ from HEAD or origin/main has not
 * yet advanced to this exact commit.
 */
export function assertReleaseCheckoutReady(sourceCommit, run = spawnSync) {
  const expected = normalizeBuildSha(sourceCommit, 'checked-out candidate commit');
  const status = run('git', ['status', '--porcelain=v1', '--untracked-files=no'], { encoding: 'utf8' });
  if (status.status !== 0) {
    const detail = status.stderr?.trim() || 'git status failed';
    throw new Error(`cannot inspect the release checkout: ${detail}`);
  }
  if (status.stdout.trim()) {
    throw new Error('release checkout has uncommitted tracked changes; commit the final handoff before deployment');
  }

  const remote = run('git', ['rev-parse', '--verify', 'refs/remotes/origin/main'], { encoding: 'utf8' });
  if (remote.status !== 0) {
    const detail = remote.stderr?.trim() || 'origin/main is unavailable';
    throw new Error(`cannot resolve published origin/main: ${detail}`);
  }
  const published = normalizeBuildSha(remote.stdout.trim(), 'published origin/main commit');
  if (published !== expected) {
    throw new Error(`checked-out candidate ${expected} must be pushed to origin/main before deployment; origin/main is ${published}`);
  }
  return expected;
}

/**
 * A full immutable tag alone is not enough. Deploying a previous full tag can
 * still leave the current candidate unserved, so bind the requested image to
 * the release revision that the production claim will later verify.
 */
export function assertDeploymentImageMatchesSource(image, sourceCommit) {
  const imageCommit = buildShaFromImage(image);
  const expectedCommit = normalizeBuildSha(sourceCommit, 'checked-out candidate commit');
  if (imageCommit !== expectedCommit) {
    throw new Error(`deployment image build SHA must match the checked-out candidate commit; expected ${expectedCommit}, got ${imageCommit}`);
  }
  return imageCommit;
}
