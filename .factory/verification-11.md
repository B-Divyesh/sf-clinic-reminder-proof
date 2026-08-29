# Independent product verification 11 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-11`
Candidate: `4b07dd38cae3bb33530eca8704aff3f9b243cbfb`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The candidate code, local quality gates, first-read gate, demo, accessibility automation, performance, and live build identity pass. The production topology does not match the checked-in one-replica durable-storage contract. It is currently running three replicas without the durable or backup mounts. This splits both managed clinic state and rate-limit state between processes and makes managed records container-local.

The earlier deployment-only failure is therefore reproduced from fresh evidence. A second, medium-severity mobile touch-target defect also remains.

## First-read and demo gate

**PASS.** A cold 1440×900 production visit with a fresh browser profile showed, before scrolling:

- What it does: **“See every reminder outcome.”**
- For whom: independent clinics needing delivery proof and a clear next step after a reminder fails.
- What to do first: the visible **“Try it with sample data”** action.
- What happens next: **“Opens a sample clinic. Nothing touches real clinic data.”**

The one-click action opened `/demo` with five realistic fictional appointments and the persistent **“Demo — sample data, nothing is saved to your clinic”** banner, **Reset demo**, and **Start for real**.

## Mandatory claims gate

`.factory/claims.json` exists with 31 entries. From the clean candidate checkout I ran `npm ci`, then every listed `test` command separately and in manifest order before the broader audit. The clean local result was **31/31 PASS**:

| Claims | Local result |
| --- | --- |
| `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline` | PASS |
| `exception-ownership`, `sample-exception-visibility`, `demo-reset`, `minimal-reminder-content`, `public-price` | PASS |
| `demo-cookie-lifetime`, `demo-replica-continuity`, `no-tracking`, `explicit-theme-choice`, `request-protection` | PASS |
| `rate-limit-policy`, `security-headers`, `build-identity`, `managed-auth-storage`, `signed-calendar-intake` | PASS locally |
| `approved-whatsapp-dispatch`, `twilio-receipt-verification`, `resend-receipt-verification`, `managed-secret-encryption`, `managed-data-minimisation` | PASS |
| `no-marketing-campaigns`, `signed-in-export-delete`, `managed-provider-fallback-receipt`, `managed-billing-return`, `managed-storage-recovery`, `single-replica-durable-topology` | PASS locally |

The local topology claim reads checked-in configuration. It does not prove that configuration was applied to production.

An initial aggregate live browser run completed 39/39 while production was under lighter load. After production scaled out, a fresh isolated run of the exact declared command failed:

```text
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in \
  npm run test:e2e -- --grep @claim:rate-limit-policy

Expected: 429
Received: 200
```

The trace showed request six going to production and returning 200. The live failure is repeatable and agrees with the Azure topology below, so it is release-blocking.

## Findings

### QA11-01 — Critical — production has three state owners and no durable storage mounts

Read-only Azure inspection returned:

```json
{
  "revision": "sf-clinic-reminder-proof--0000037",
  "active": true,
  "healthState": "Healthy",
  "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:4b07dd38cae3",
  "replicas": 3,
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "mounts": null
}
```

This contradicts `deployment/containerapp.json`, which requires `minReplicas=1`, `maxReplicas=1`, and separate Azure Files mounts at `/durable` and `/backups`.

Observed live rate-limit behavior with one stable first `X-Forwarded-For` hop:

- Requests 1–15 to `POST /api/v1/demo/workspaces`: 200.
- Request 16: 429 JSON, `code: rate_limited`, `Retry-After: 3599`.
- The documented and claimed allowance is five creations per client per hour, so request six should have been rejected.
- `X-RateLimit-Remaining` advanced in three independent sequences, matching the three active replicas.
- A 150-request concurrent `/metrics` probe returned 135×200 and 15×429, with `Retry-After: 1`, further demonstrating process-local general limits.

Managed SQLite, generated encryption keys, durable snapshots, and recovery pairs are also process-local because neither Azure Files mount exists. A clinic can be routed to a replica that does not hold its workspace/key, and all such data can disappear on replacement. This invalidates the live managed persistence, recovery, isolation, and rate-limit promises.

Required repair: deploy a new revision with exactly one replica and both checked-in Azure Files mounts, confirm 100% traffic on it, then prove request six returns 429 and a managed fixture survives replica replacement and backup restore. Do not accept real clinic data until that succeeds.

### QA11-02 — Medium — core mobile evidence links are below the 44 px touch-target minimum

At a 390×844 viewport, each of the five **View evidence** links in the demo ledger measured **92×18 CSS px**. Other checked public-route controls met 44 px, and the rows have room to enlarge these links. The product accessibility contract and attached baseline require every interactive target to be at least 44×44 px.

Required repair: give each ledger evidence link a minimum 44 px block or inline-flex hit area, retain its visible focus treatment, and add a mobile assertion covering all demo controls rather than footer links only.

## Local quality gates

| Check | Result |
| --- | --- |
| Candidate checkout | PASS — clean `main`, exact SHA `4b07dd38cae3bb33530eca8704aff3f9b243cbfb` |
| `npm ci` | PASS — 87 locked packages, zero audit vulnerabilities |
| Every `.factory/claims.json` command | PASS locally — 31/31 |
| `npm test` | PASS — 9 Vitest, 33 Rust, 39 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/0 warnings, rustfmt clean, Clippy with warnings denied |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` emitted |
| Release runtime with only `PORT=4811` | PASS — `/health` returned build `dev`; startup reported generated local key without exposing it |
| Concurrent local health smoke | PASS — 100/100 requests returned 200 |
| Container build | Not run — neither Docker nor Podman is installed in the verifier container |

The production-style web rebuild with `BUILD_SHA=4b07dd38…` emitted `/assets/index-iloOFuty.js`. Its SHA-256 exactly matched the live asset (`e4695571…afc1`); the live CSS also matched (`0b1797c6…e0f3`). Live `/health` returned the full candidate SHA.

## End-to-end behavior and recovery

- The seeded demo represents delivered SMS, rejected approved WhatsApp with email fallback, consent opt-out, delivered email plus reply, and a cancelled appointment.
- Advancing due reminders produced delivery evidence or a staff exception; Jordan L.'s timeline showed `TEMPLATE_REJECTED` followed by simulated email delivery.
- Sofia R.'s exception could be assigned, resolved, persisted across reload, undone, and reset to the canonical seed.
- Malformed JSON returned 400; wrong content type returned 415; a 17 KB body returned 413; anonymous checkout returned 401 with `WWW-Authenticate: Bearer`. All were structured JSON with a matching UUID request ID and actionable copy.
- Deep links, browser Back, styled HTTP 404, offline read state, disabled offline writes, and 200% text reflow passed in the live browser suite.

The authenticated managed workflow was not exercised with real clinic/provider credentials or patient data. Local signed fixtures cover connector signatures, receipt signatures/replays, consent/fallback, tenant ownership, encryption, export/delete, billing return, and backup restore. The missing live mounts prevent those local persistence results from being accepted as production evidence.

## Privacy, security, authentication, and accessibility

- A fresh landing-to-demo interaction logged 19 requests, all to `https://clinic-reminder-proof.sociobot.in`; no analytics, messaging provider, billing, or third-party runtime request occurred. There were no console, page, or failed-request errors.
- Demo cookies are HttpOnly, Secure, SameSite=Lax, scoped to `/api/v1/demo`, and have `Max-Age=86400`.
- Root responses include CSP with response-header `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP.
- The sign-in action reached only the required Sociobot Microsoft Entra authority, tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client `25c704f4-465a-47af-80ab-2c489466b697`, production callback, and PKCE. The tenant displayed its sign-in/create-account page without a redirect-registration error.
- The shipped Playwright axe integration found zero serious/critical issues across public routes in light and dark themes. An independent 390 px dark/reduced-motion demo check also found zero serious/critical issues, no horizontal overflow, and no active motion.
- Keyboard smoke: first Tab exposed the skip link with a 3 px `rgb(0, 95, 204)` focus outline; Enter moved focus to `<main>`. QA11-02 remains because touch size is a separate failure.
- `/opt/fleet/lib/verify-url.sh` passed: 200, title, `lang=en`, one H1, main landmark, alt/control labels, and zero console errors.

## Performance, caching, and bundle budgets

Fresh mobile Lighthouse on production:

| Category/metric | Result |
| --- | ---: |
| Performance | 99 |
| Accessibility | 100 |
| Best practices | 100 |
| SEO | 100 |
| FCP | 1.41 s |
| LCP | 1.50 s |
| TBT | 115 ms |
| CLS | 0.00074 |

Live initial transfer sizes were 28,286 bytes gzip for JS and 5,499 bytes gzip for CSS. The lazy MSAL chunk is not loaded on landing/demo. Fonts total under 120 KB. HTML is `no-cache`; hashed JS/CSS are `public, max-age=31536000, immutable`.

## Applicability notes

Reminder Proof is a web product with a backend, not a library, CLI, or PWA. Consumer-package, CLI-install, service-worker update, and offline-reload-PWA checks do not apply.

## Defects by severity

| Severity | Findings |
| --- | --- |
| Critical | QA11-01: live three-replica/no-volume topology splits and loses managed state and multiplies rate allowances |
| High | None |
| Medium | QA11-02: five core demo evidence links are only 92×18 px on mobile |
| Low | None |

No product code was modified during verification.
