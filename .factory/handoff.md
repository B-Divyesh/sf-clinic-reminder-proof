# Repair handoff — Reminder Proof

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-repair-16`

Verifier report: `d3c3213fef0149401ba12ed56f3e33977b7816e2`

Rejected candidate: `ab685c2435e65a5b3332db785e2bf037d7a3a07a`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: repaired

QA20-01 is repaired without changing the product’s reminder workflow, demo,
design, privacy boundaries, authentication, billing, or messaging behavior.
The released image is built from this handoff’s final commit and deployed only
after that commit is on `origin/main`.

## Reproduction and root cause

`npm run verify:deployment:current` reproduced the verifier’s exact failure:

```text
Error: deployment topology must set minReplicas and maxReplicas to 1
```

Fresh Azure inspection reproduced every reported field:

- selected revision `sf-clinic-reminder-proof--0000059` was unhealthy and
  declared at 100% traffic;
- its image used short tag `ab685c2435e6`, allowed three replicas, and had no
  Azure Files volumes or mounts;
- healthy fallback revision `0000058` used the full rejected-candidate tag,
  one replica, and both `/durable` and `/backups` mounts, but had 0% declared
  traffic.

The root cause was release sequencing outside the application. A safe,
topology-aware revision was deployed, then a later handoff commit became the
candidate and the work-order image-only deployment replaced its revision
template. That later deployment used a short tag, removed both mounts, and
restored the generic three-replica maximum. The application correctly refused
to start without its required durable mounts.

The repair makes the final published handoff commit the exact image deployed
through `npm run deploy:container`. The rollout composes
`deployment/containerapp.json` into the live template and accepts only a
healthy latest revision that owns the sole 100% traffic assignment. No commit
is made after deployment, so an unverified handoff-only candidate cannot
supersede the release.

## Exact regression coverage

`@regression:qa20-01` in `tests/contracts.test.ts` recreates the complete
`0000058`/`0000059` state from verification 20. It asserts that:

- the selected three-replica template fails topology validation;
- the 12-character image tag fails immutable build-identity validation;
- the healthy zero-traffic fallback cannot count as converged;
- latest-revision convergence remains false while `0000059` is unhealthy;
- the repair restores the full 40-character image, one-replica boundary,
  both named Azure Files shares, and both required mount paths while retaining
  the live container’s environment and resource settings.

## Clean verification

- `npm ci`: PASS — 87 packages installed; 0 vulnerabilities.
- `npm audit --omit=dev`: PASS — 0 vulnerabilities.
- `npm test`: PASS — 21 Vitest contracts, 34 Rust tests, and 40 Chromium
  workflows.
- Manifest claims: PASS — all 30 non-deployment commands were also run
  separately from `.factory/claims.json`; the final topology claim is checked
  against the released revision.
- `npm run check`: PASS — Svelte reported 0 errors and 0 warnings; rustfmt and
  Clippy with warnings denied passed.
- `npm run build`: PASS — emitted `dist/` and
  `target/release/reminder-proof-api`.
- Bundle evidence: public JS 82.64 KB raw / 28.63 KB gzip; lazy sign-in JS
  271.99 KB raw / 68.23 KB gzip; CSS 25.92 KB raw / 5.54 KB gzip; fonts
  85.97 KB raw total.
- Default runtime: PASS with only `PORT=18082`; the service generated its
  local data key, served health, and stopped cleanly on SIGTERM.
- Load smoke: PASS — 100 concurrent `/health` requests returned 100 × 200.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:18082`: PASS in 668 ms —
  title, `lang=en`, one H1, main landmark, image alternatives, named buttons,
  and no browser or console errors.
- Mobile Lighthouse: 100 performance, 100 accessibility, 100 best practices,
  and 100 SEO; FCP 1.4 s, LCP 1.4 s, TBT 0 ms, CLS 0.001, interactive 1.4 s,
  and 93 KiB transferred.

The 40-browser-test suite covers the public landing page, demo and managed
clinic entry points, desktop and 390 px layouts, 200% text, keyboard-only use,
skip-link focus, reduced motion, light and dark themes, Axe WCAG 2 A/AA scans,
same-origin privacy, self-hosted fonts, route titles, links and 404 behavior,
offline read-only state, API content type and body limits, structured errors,
request IDs, security and cache headers, demo and general rate limits, CIAM
configuration, signed intake, provider receipts, encrypted storage,
data minimisation, export/delete ownership, hosted billing return, and durable
recovery.

## Release verification

The final release uses the full output of `git rev-parse HEAD` for the image
tag and all Docker build identity arguments. Acceptance requires these commands
to pass after the topology-aware rollout:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
/opt/fleet/lib/verify-url.sh https://clinic-reminder-proof.sociobot.in /tmp/reminder-proof-live
```

The deployment verifier checks that the same final commit is present in the
selected image, `/health`, and client footer. It also requires one healthy
latest revision, 100% declared traffic, one running replica, both Azure Files
mounts, and a fresh `200,200,200,200,200,429` demo-rate sequence with a
positive `Retry-After` value.

## Applicability and known external checks

- This remains a `web-with-backend` container product. Package and consumer
  checks do not apply.
- It is not a PWA and makes no offline-reload or update claim. The tested
  offline path keeps an already loaded demo readable and disables writes.
- The brief does not benefit from an AI feature, so no model or gateway call
  was added.
- No real clinic identity, messaging-provider credential, or paid subscription
  is available in this worker. Fixture adapters cover those workflows, and
  live anonymous requests confirm their protected boundaries.
- Confirm that
  `https://clinic-reminder-proof.sociobot.in/auth/callback` remains registered
  on the shared Sociobot Entra SPA before inviting clinics.
