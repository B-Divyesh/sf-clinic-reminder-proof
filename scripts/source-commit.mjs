import { spawnSync } from 'node:child_process';
import { normalizeBuildSha } from './deployment-identity.mjs';

/**
 * Resolve the immutable revision being verified. Claims run from a fresh
 * checkout, so deriving HEAD prevents a previous release's SHA from silently
 * becoming the deployment acceptance criterion.
 */
export function resolveCheckedOutSourceCommit(run = spawnSync) {
  const result = run('git', ['rev-parse', '--verify', 'HEAD'], { encoding: 'utf8' });
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || 'git rev-parse failed';
    throw new Error(`cannot resolve the checked-out source commit: ${detail}`);
  }
  return normalizeBuildSha(result.stdout.trim(), 'checked-out source commit');
}
