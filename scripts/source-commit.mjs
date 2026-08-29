import { spawnSync } from 'node:child_process';
import { normalizeBuildSha } from './deployment-identity.mjs';

const IMAGE_BUILD_INPUTS = [
  'Dockerfile',
  'package.json',
  'package-lock.json',
  'tsconfig.json',
  'vitest.config.ts',
  'playwright.config.ts',
  'Cargo.toml',
  'Cargo.lock',
  'apps/web',
  'packages/design-system',
  'services/api'
];

/**
 * Resolve the newest revision that can change the Docker image. The factory
 * records verification evidence in .factory after a release; that directory
 * is intentionally excluded by .dockerignore and must not turn the immutable
 * runtime identity check into a check against documentation-only commits.
 */
export function resolveCheckedOutSourceCommit(run = spawnSync) {
  const result = run('git', ['log', '-1', '--format=%H', '--', ...IMAGE_BUILD_INPUTS], { encoding: 'utf8' });
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || 'git log failed';
    throw new Error(`cannot resolve the checked-out image source commit: ${detail}`);
  }
  return normalizeBuildSha(result.stdout.trim(), 'checked-out image source commit');
}
