# Verification handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-18`
Candidate: `36d39a8d57aa77e3d8131b5e0359d22d9519883e`
Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL — release blocked

Candidate code passes local install, tests, checks, production build, core demo,
accessibility, privacy, security-header, rate-limit, persistence, and performance
verification. The public fallback also serves the exact candidate build.

Release still fails because Azure declares unhealthy revision
`sf-clinic-reminder-proof--0000053` at 100% traffic. That revision uses the short
tag `36d39a8d57aa`, permits three replicas, and has neither required Azure Files
volume or mount. Public requests are falling back to healthy full-SHA revision
`0000052`, which Azure reports at 0% traffic. The mandatory
`single-replica-durable-topology` claim therefore fails.

The full independent record is in [verification-18.md](verification-18.md).

## Defects

- **Critical — QA18-01:** unhealthy 100%-traffic revision has unsafe topology
  and makes `npm run verify:deployment:current` fail.
- **Medium — QA18-02:** the live rate-limit claim test reuses predictable client
  keys for one hour, so repeat runs can begin with 429. A random fresh key
  independently confirmed the correct five-allowed, sixth-rejected behavior.

## Verification summary

- Mandatory claims: 30 PASS, 1 FAIL (`single-replica-durable-topology`).
- First-read and one-click demo: PASS on desktop and 390 px mobile.
- `npm ci`: PASS, 87 packages, 0 vulnerabilities.
- `npm test`: PASS, 18 Vitest + 34 Rust + 40 Chromium.
- `npm run check`: PASS, no Svelte, rustfmt, or Clippy findings.
- `npm run build`: PASS; `dist/` and release API emitted.
- Live Playwright: 39/40; only the repeat-client rate test failed.
- Axe: zero serious/critical findings in light and dark across public routes.
- Factory `verify-url.sh`: PASS, no console/page errors.
- Lighthouse mobile: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.45 s, TBT 151 ms, CLS 0.00074.
- Public health/footer: exact candidate SHA / `Build 36d39a8`.
- Full-SHA local web build matches the live entry JavaScript byte-for-byte.
- Demo limit observed: five per client per hour; sixth 429 with Retry-After.
- General API burst observed: 40 concurrent allowed, excess requests 429 with
  Retry-After. Health is exempt.
- Docker image build was not run because this worker has no Docker CLI.

## Repair and recheck

Roll out the full 40-character candidate image using the checked-in
topology-aware deploy command. Require one healthy 100%-traffic revision,
`minReplicas=maxReplicas=1`, `/durable` and `/backups` Azure Files mounts, and
matching health/footer identity. Randomize the live rate-limit test client key.

Then run:

```sh
npm ci
npm test
npm run check
npm run build
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Needs operator action

- Confirm `https://clinic-reminder-proof.sociobot.in/auth/callback` remains
  registered on the shared Sociobot Entra SPA before inviting clinics.
- If credentials are available during the next verification, complete one test
  clinic sign-in and Sociobot subscription checkout. Fixture coverage passed;
  this verifier did not use a real identity, provider credential, or payment.
