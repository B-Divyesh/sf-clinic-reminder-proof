# Independent verification 16 — FAIL

Date: 2026-08-29 UTC
Candidate: `4a23898a51958f530909bfe9b3e86678403ad86d`
URL: <https://clinic-reminder-proof.sociobot.in>

## Release decision

**FAIL.** The production candidate revision is unhealthy and cannot serve the candidate. Its declared 100% traffic revision has neither required durable Azure Files mount and permits three replicas. Public requests are transparently served by the previous healthy build, `659284a`, rather than the candidate.

## First-read test

A fresh browser with no prior storage loaded the landing page successfully. It plainly says it shows independent clinics every appointment-reminder outcome, names its intended users, and puts **Try it with sample data** on the first screen. The adjacent text says that it opens a sample clinic and does not touch real clinic data. This part passes. Evidence: `qa-artifacts/verification-16-live-cold.png`.

## Required claims

`.factory/claims.json` exists and contains 31 claims. After `npm ci`, I ran each literal `test` command from the manifest from the supplied clean checkout through the demo entry point. The local browser and Rust claim assertions pass; the final required deployment command fails:

```text
npm run test:e2e -- --grep @claim:single-replica-durable-topology
  1 passed
npm run verify:deployment:current
  Error: deployment topology must set minReplicas and maxReplicas to 1
```

This is release-blocking under the claims contract. The final command fails before it can accept production identity. The complete local browser suite exercised 40 tests; the local topology-only assertion passes because the checked-in template has one replica and both mounts, while the production control plane does not.

## Local verification

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages installed, 0 vulnerabilities |
| `npm test` | PASS locally — 15 Vitest tests, 34 Rust tests, 40 Chromium tests |
| `npm run check` | PASS — Svelte: 0 errors/warnings; rustfmt and Clippy clean |
| `npm run build` | PASS — Vite `dist/` and `target/release/reminder-proof-api` produced |
| Bundle budget | PASS — local initial JS 96.86 KB gzip (28.63 + 68.23), CSS 5.54 KB gzip, loaded WOFF2 fonts 55.31 KB |

The product is a web service, not a library/CLI or PWA; package-consumer and service-worker update checks do not apply.

## Live product checks (healthy fallback build only)

- Cold landing, desktop demo, and 390 px mobile demo worked. No horizontal overflow at 390 px. Demo banner contains Reset demo and Start for real.
- Demo normal flow advanced reminders and exercised assignment, resolution, reset, and recovery in the claims suite. Invalid unauthenticated demo state returned 401 JSON with a matching `X-Request-Id`.
- Playwright Axe found no serious or critical findings on landing or demo at desktop and 390 px. Console/page errors were zero. Keyboard Tab reached the skip link, navigation, theme control, and demo controls; visible focus was a 3 px `#005fcc` outline. Reduced-motion animation/transition duration was effectively zero (`0.00001s`).
- Cold landing and landing-to-demo request logs contained only this origin, including self-hosted fonts. No tracker or third-party runtime request appeared. The demo cookie and data requests stayed same-origin.
- `/api/v1/auth/config` advertises only the mandated Entra tenant and `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`.
- Public HTML has CSP, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, COOP, and `frame-ancestors 'none'`. Hashed assets return `Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.
- Rate-limit observation: six `POST /api/v1/demo/workspaces` requests with one stable first `X-Forwarded-For` hop returned `200,200,200,200,200,429`; the sixth returned `Retry-After: 3599`. Observed allowance: five demo workspace creations per client per hour.

Screenshots: `qa-artifacts/verification-16-live-cold.png`, `qa-artifacts/verification-16-live-demo-desktop.png`, `qa-artifacts/verification-16-live-demo-mobile.png`, and `qa-artifacts/verification-16-live-landing-390.png`.

## Deployment identity and failure evidence

Public `/health` and the footer both report `659284a4195b954267ae7f243c129633963a24f6`, not candidate `4a23898…`.

| Revision | Image | Health / traffic | Topology |
| --- | --- | --- | --- |
| `sf-clinic-reminder-proof--0000048` | full `659284a4195b954267ae7f243c129633963a24f6` | Healthy / 0% declared traffic | one replica, both Azure Files volumes |
| `sf-clinic-reminder-proof--0000049` | `4a23898a5195` | **Unhealthy / 100% declared traffic** | `minReplicas: 1`, **`maxReplicas: 3`**, no volumes or mounts |

Candidate revision logs give the direct cause:

```text
initialize durable clinic store: "required durable storage mounts are missing:
/durable, /backups; refusing unsafe production storage"
```

The candidate correctly refuses to start without durable storage, but the factory deployment did not apply its checked-in topology. The ingress therefore falls back to the older healthy revision, which explains why the public UI looks healthy while candidate verification fails.

## Defects by severity

### V16-01 — Critical — candidate is not serving live

`/health` and footer identify `659284a…`; the candidate's `0000049` revision is unhealthy. The requested candidate and live deployment do not match.

### V16-02 — Critical — candidate revision violates durable single-replica topology

Revision `0000049` has no `/durable` or `/backups` mounts and declares `maxReplicas: 3`. If it were made to start without the guard, it could split the process-local limiter and durable storage owner across replicas. It fails the mandatory `single-replica-durable-topology` claim.

No separate product UX, accessibility, privacy-request, authentication, or bundle-budget defects were reproduced on the older fallback build.

## Required remediation and re-test

1. Deploy the full `4a23898a51958f530909bfe9b3e86678403ad86d` image through `scripts/deploy-containerapp.mjs`, preserving `minReplicas=maxReplicas=1` and both Azure Files mounts.
2. Wait for that revision to become healthy and be the sole 100% traffic target; do not rely on ingress fallback.
3. Confirm `/health` and the footer report the exact 40-character candidate SHA.
4. Re-run every claims command and `npm run verify:deployment:current` from a clean checkout.
