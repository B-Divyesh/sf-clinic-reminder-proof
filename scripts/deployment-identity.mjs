const FULL_COMMIT_SHA = /^[a-f0-9]{40}$/i;

export function normalizeBuildSha(value, description = 'build SHA') {
  if (typeof value !== 'string' || !FULL_COMMIT_SHA.test(value)) {
    throw new Error(`${description} must be a full 40-character Git commit SHA`);
  }
  return value.toLowerCase();
}

export function buildShaFromImage(image) {
  if (typeof image !== 'string' || !image.trim()) {
    throw new Error('container image is required');
  }

  const tag = image.slice(image.lastIndexOf(':') + 1);
  if (tag === image || image.includes('@')) {
    throw new Error('container image must use an immutable full-commit tag, not a digest or an untagged reference');
  }
  return normalizeBuildSha(tag, 'container image tag');
}

export function assertImageMatchesBuildSha(image, expectedBuildSha) {
  const expected = normalizeBuildSha(expectedBuildSha);
  const imageBuildSha = buildShaFromImage(image);
  if (imageBuildSha !== expected) {
    throw new Error(`container image tag must equal expected build SHA ${expected}, got ${imageBuildSha}`);
  }
  return expected;
}

export function assertPublicBuildIdentity({ healthBody, frontEndSource }, expectedBuildSha) {
  const expected = normalizeBuildSha(expectedBuildSha);
  if (healthBody?.build_sha !== expected) {
    throw new Error(`live health build identity; expected ${expected}, got ${healthBody?.build_sha ?? 'none'}`);
  }

  // The footer is rendered by the client-side Svelte app. The static HTML
  // shell therefore has no footer text; its module must contain both the
  // full immutable SHA and the footer's Build rendering code.
  if (typeof frontEndSource !== 'string' || !frontEndSource.includes(expected) || !frontEndSource.includes('Build ')) {
    throw new Error(`live landing footer build identity; expected the public app bundle to render Build ${expected.slice(0, 7)}`);
  }
  return expected;
}

export async function fetchPublicBuildIdentity(liveUrl, expectedBuildSha, request = fetch) {
  const health = await request(`${liveUrl}/health`);
  if (!health.ok) throw new Error(`live health returned ${health.status}`);
  const healthBody = await health.json();

  const landing = await request(`${liveUrl}/`);
  if (!landing.ok) throw new Error(`live landing returned ${landing.status}`);
  const landingHtml = await landing.text();
  const moduleUrls = [...landingHtml.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["']/gi)]
    .map((match) => new URL(match[1], liveUrl).toString());
  if (!moduleUrls.length) throw new Error('live landing has no public application module');
  const modules = await Promise.all(moduleUrls.map(async (url) => {
    const response = await request(url);
    if (!response.ok) throw new Error(`public application module returned ${response.status}`);
    return response.text();
  }));

  return assertPublicBuildIdentity({ healthBody, frontEndSource: modules.join('\n') }, expectedBuildSha);
}
