# Verification handoff — Reminder Proof

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-verify-20`

Candidate: `ab685c2435e65a5b3332db785e2bf037d7a3a07a`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL

Independent verification is complete. The product source, candidate identity,
demo workflow, local gates, live browser suite, accessibility, privacy,
security, rate limits, and performance all passed. Release is blocked because
the required `single-replica-durable-topology` claim fails against Azure.

Azure currently declares unhealthy revision
`sf-clinic-reminder-proof--0000059` at 100% traffic. It uses short image tag
`ab685c2435e6`, permits three replicas, and has no durable or backup volumes or
mounts. It exits with:

```text
required durable storage mounts are missing: /durable, /backups; refusing
unsafe production storage
```

The public URL works through healthy fallback revision `0000058`, which uses
the full candidate SHA, one replica, and both required Azure Files mounts, but
Azure reports it at 0% declared traffic. `/health` and the live asset bytes
match the candidate; the required selected serving topology does not.

## Release-blocking defect

| Severity | ID | Finding |
| --- | --- | --- |
| Critical | QA20-01 | Selected revision `0000059` is `Unhealthy` / `ActivationFailed`, declared at 100% traffic, with `maxReplicas: 3` and no `/durable` or `/backups` mounts. |

## Verification summary

- Claims: 30/31 passed. `single-replica-durable-topology` failed in the exact
  required `npm run verify:deployment:current` command.
- First-read: PASS on desktop and 390 px; what it does, audience, first action,
  and one-click sample demo are all above the fold.
- `npm ci`: PASS, 87 packages; `npm audit --omit=dev`: 0 vulnerabilities.
- `npm test`: PASS — 20 Vitest, 34 Rust, 40 Playwright.
- `npm run check`: PASS — Svelte, rustfmt, and Clippy.
- Exact `npm run build`: PASS — emitted `dist/` and release API.
- Live Playwright: 40/40 passed.
- Axe: zero serious/critical findings on all public/app-entry routes in both
  themes.
- `verify-url.sh`: PASS; no console errors or missing baseline semantics.
- Lighthouse mobile: 99 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.4 s, CLS 0.001, 89 KiB transfer.
- Demo rate allowance: five creates per client per hour; request six returned
  429 with `Retry-After: 3599`. General and protected endpoints also returned
  429 with positive `Retry-After` after their burst allowance.
- Runtime: starts with only `PORT`, generates a data key, survives 100
  concurrent health requests, and shuts down cleanly.

Full evidence, workflow coverage, boundary results, deployment inspection,
bundle sizes, and limitations are in
[verification-20.md](verification-20.md).

## Required next action

Deploy only through the checked-in topology-aware rollout using the full
40-character candidate tag. Require one healthy latest revision at the sole
100% traffic target, `minReplicas=maxReplicas=1`, and both Azure Files mounts:

```sh
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:ab685c2435e65a5b3332db785e2bf037d7a3a07a
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

Do not accept the release until both verification commands pass from a fresh
checkout.

## Known external check

No real Entra user, messaging-provider credential, or paid Sociobot
subscription was available. The required CIAM redirect and all protected live
boundaries passed; fixture adapters cover provider, billing, and persistence
logic. Confirm the production callback registration before inviting clinics.
