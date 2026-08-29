# Independent product verification 9 — PASS

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-9`

Candidate: `9d0d5c31576150d5cfa96b069081c6a2d690e33c`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**PASS — release candidate accepted.** No release-blocking defects remain.

The deployed service reported the exact candidate identity:

```json
{"status":"ok","build_sha":"9d0d5c31576150d5cfa96b069081c6a2d690e33c"}
```

This independently supersedes the earlier deployment-only failure: a live demo persisted through advance, assignment, resolution, and undo, and the deployment now reports the requested SHA.

## Claims gate

`.factory/claims.json` exists and contains 25 claims. After `npm ci` (87 packages; 0 vulnerabilities), I ran every command in that manifest exactly, one at a time. **25/25 passed.** The demo-entry browser claims began from the shipped demo route and data.

| Claims | Result |
| --- | --- |
| `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline` | PASS |
| `exception-ownership`, `sample-exception-visibility`, `demo-reset`, `minimal-reminder-content`, `public-price` | PASS |
| `demo-cookie-lifetime`, `demo-replica-continuity`, `no-tracking`, `explicit-theme-choice`, `request-protection` | PASS |
| `rate-limit-policy`, `security-headers`, `build-identity`, `managed-auth-storage`, `no-marketing-campaigns` | PASS |
| `signed-in-export-delete`, `managed-provider-fallback-receipt`, `managed-billing-return`, `managed-storage-recovery`, `single-replica-durable-topology` | PASS |

The full quality suite also passed: `npm test` reported 7 Vitest tests, 27 Rust tests, and 33 Chromium tests. A preliminary all-at-once Playwright-only run timed out one managed claim while two workers compiled the Rust test target. The same claim passed in an isolated cold target (3 minutes overall; 16 seconds after the web server was ready), then passed again in the exact manifest run and in `npm test`; it is not a reproducible product or claim failure.

## Build and code-quality gates

- `npm run check`: PASS — Svelte reported 0 errors and 0 warnings; `cargo fmt --check` and Clippy with `-D warnings` passed.
- `npm run build`: PASS — emitted `dist/` and `target/release/reminder-proof-api`.
- Production initial payload observed cold: 28,196 B compressed JavaScript, 5,491 B CSS, 25,254 B self-hosted font, and 364 B HTML (59,841 B total resources). This is below the 200 KB initial-JS and 50 KB CSS budgets. The 271,994 B MSAL chunk is lazy rather than requested on the landing load.
- Hashed assets use `Cache-Control: public, max-age=31536000, immutable`; documents use `no-cache`.

## First-read, functional, and responsive checks

Cold production desktop first read passed. The first screen says:

- What: “See every reminder outcome.”
- For whom: independent clinics needing delivery proof and a next step after a reminder fails.
- First action: “Try it with sample data,” with the adjacent explanation that it opens a sample clinic and does not touch real clinic data.

The one-click `/demo` / `?demo=1` sandbox has the persistent “Demo — sample data, nothing is saved to your clinic” banner, Reset demo, and Start for real actions. With a fresh client, live sample data advanced to 4 due reminders, 3 delivered outcomes, and 1 staff-owned exception. I assigned Sofia R. to Sam Rivera, resolved it as Called patient, and undid the resolution. All five demo API responses were 200.

At 390 px the landing and demo reflowed without horizontal overflow; visible interactive controls measured at least 44 px high. Keyboard traversal reached the skip link, navigation, theme selector, demo controls, and footer in order. Focus on the sample-data link is a visible `rgb(0, 95, 204)` 3 px outline with 3 px offset. Reduced-motion emulation produced no running animations.

The live axe scan on the normal 390 px demo found **zero serious or critical violations**. The full browser suite separately passed public-route axe, landmarks, one-h1/title metadata, link crawl, HTTP 404, offline handling, 200% text, keyboard operation, and reduced motion checks.

## Privacy, security, and backend checks

- Cold landing request log: only the same-origin HTML, JS, CSS, and self-hosted font.
- Full live demo request log: only same-origin page/assets and `/api/v1/demo/*`; no analytics, CDN fonts, messaging provider, billing, or AI request. It had no page errors or console errors in the normal flow.
- Demo cookie: `HttpOnly`, `Secure`, `SameSite=Lax`, path `/api/v1/demo`, with a 24-hour expiry.
- Browser responses include CSP with `frame-ancestors 'none'`, HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, restrictive permissions policy, and COOP.
- `/api/v1/auth/config` returns only the required Sociobot Entra External ID authority `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/` and shared client ID. An unauthenticated clinic request returned `401` plus `WWW-Authenticate: Bearer`.
- `/metrics` is machine-readable Prometheus text. `/health` is 200 and identifies this exact build. An unknown route returns HTTP 404 with the styled recovery view.
- Demo-create allowance observed live: 5 creates succeeded, request 6 returned `429` with `Retry-After: 3599`. Protected API allowance observed live: the limit header is 40; requests past the burst returned `429` with `Retry-After: 1`. This confirms rate enforcement and the required retry header on both public-demo and protected boundaries.
- Persistence, tenant scoping, durable recovery pair, connector/provider boundaries, receipt fallback, export/delete ownership, invalid body/content-type handling, and billing-return boundaries are covered by the passed Rust and claim tests. No real clinic, patient, credential, provider dispatch, payment, or destructive customer deletion was attempted.

## Defects by severity

No open defects found.

## Reproduce

```sh
npm ci
npm test
npm run check
npm run build
```

Use `https://clinic-reminder-proof.sociobot.in/?demo=1` for the public sandbox.
