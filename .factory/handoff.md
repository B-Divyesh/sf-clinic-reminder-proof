# Repair handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-repair-14`
Base candidate: `36d39a8d57aa77e3d8131b5e0359d22d9519883e`
Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: repaired and ready for topology-aware rollout

This repair preserves the already-passing demo, managed clinic, privacy,
accessibility, and delivery-evidence behavior. The independent source report is
[verification-18.md](verification-18.md).

## Reproduced findings and repair

- **QA18-01:** before rollout, `npm run verify:deployment:current` failed with
  exactly `deployment topology must set minReplicas and maxReplicas to 1`.
  Azure showed `0000053` at 100% traffic with short tag `36d39a8d57aa`,
  `minReplicas: 1`, `maxReplicas: 3`, and no volumes or `/durable` and
  `/backups` mounts. Healthy full-SHA revision `0000052` was at 0% traffic.
  The checked-in topology-aware rollout will replace that revision with the
  full source tag, one replica, and both Azure Files mounts.
- **QA18-02:** the old test's `198.18.<worker>.<counter>` client was reused on
  every clean run. After one live pass, the immediate second run returned
  `429, 429, 429, 429, 429` for its five expected successes, reproducing the
  verifier's one-hour bucket collision. `scripts/fresh-client-identity.mjs`
  now creates a UUID-derived RFC 3849 documentation IPv6 client for every
  probe, while all six requests in one probe retain that first hop.
- Browser probes and `verify-production-deployment.mjs` share the new client
  generator. The claim sandbox specifies a fresh unique documentation-IPv6
  client. The `@regression:qa18-01 and qa18-02` contract rejects the reported
  unsafe revision, proves the patch restores scale and volumes, and locks the
  fresh-client requirement into the browser claim.

## Verification before rollout

- `npm ci`: PASS — 87 packages; audit reported 0 vulnerabilities.
- `npm run test:web`: PASS — 19 Vitest contracts, including QA18 regression.
- `npm test`: PASS — 19 Vitest, 34 Rust API, and 40 Chromium tests.
- `npm run check`: PASS — Svelte 0 errors/0 warnings; `rustfmt` and Clippy
  with warnings denied passed.
- `npm run build`: PASS — `dist/` and `target/release/reminder-proof-api`
  emitted. Public JS is 82.64 KB raw / 28.63 KB gzip; CSS is 25.92 KB raw /
  5.54 KB gzip.
- `npm audit --omit=dev`: PASS — 0 vulnerabilities.
- Two consecutive live runs of the rate-limit claim passed. Each asserted five
  `200` responses, then `429` with a positive `Retry-After` on request six.
- The complete Chromium suite covers desktop and 390 px mobile, keyboard and
  skip-link paths, 200% text, reduced motion, Axe serious/critical findings in
  both themes, privacy/no-tracking, offline read-only behavior, routing,
  headers, and console errors.
- Docker is unavailable in this worker. Azure Container Registry builds the
  release image from this exact commit instead.

## Rollout and final recheck

Deploy only the full 40-character source tag with the checked-in
topology-aware command. It reapplies `deployment/containerapp.json`, waits for
one healthy 100%-traffic revision, and verifies public health/footer identity.
Run this exact sequence after the clean commit is pushed:

```sh
npm ci
npm test
npm run check
npm run build
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

The final verification must report `minReplicas: 1`, `maxReplicas: 1`, both
Azure Files mounts, rate statuses `200,200,200,200,200,429`, and a positive
`Retry-After`.

## Needs operator action

- Confirm `https://clinic-reminder-proof.sociobot.in/auth/callback` remains
  registered on the shared Sociobot Entra SPA before inviting clinics.
- If credentials are available during the next verification, complete one test
  clinic sign-in and Sociobot subscription checkout. Fixture coverage passed;
  this verifier did not use a real identity, provider credential, or payment.
