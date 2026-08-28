# Reminder Proof repair handoff

Status: **repair candidate — technical QA blockers closed; regulated live dispatch remains intentionally unavailable**

Date: 2026-08-28 UTC

Work order: `clinic-reminder-proof-repair-1`

Base report: `ff6f95b2cd9d30cb3f93dff935068eefca3e19e2`, candidate `6e4cbb77f20f9668b9d0f27dc9e257eb790e6fe1`

## What changed

- Replaced process-local demo workspaces with a 59-byte self-contained cookie state. The cookie is `HttpOnly; Secure; SameSite=Lax`, scoped to `/api/v1/demo`, and expires after 24 hours. Fictional state now survives replica changes and process restarts.
- Changed all limiter keys to the ingress-appended final `X-Forwarded-For` hop. Caller-supplied leading hops no longer choose a bucket. Creation, demo writes, the general API governor, and `/metrics` return JSON `429` responses with `Retry-After`.
- Added structured JSON problems for malformed JSON and bodies over 16 KB.
- Added `/metrics`, HSTS, gzip response compression, immutable one-year caching for hashed assets, `Secure` cookies, and real HTTP 404 responses while preserving SPA deep links.
- Changed the Rust build stage to `rust:1-slim`.
- Fixed skip-link focus transfer, 44 px navigation/footer targets, loading-state mutation controls, and 200% text/mobile overflow.
- Replaced the dead “Start for real” action with `/start`: a local CSV evidence intake. It applies consent precedence, records primary and fallback provider results, creates owner fields for unresolved outcomes, persists in the browser, exports proof CSV, and deletes locally.
- Expanded `.factory/claims.json` from 9 to 17 claims. Every ID occurs in exactly one Playwright `@claim:<id>` test.

## Exact regressions for verifier findings

| Finding | Root-cause regression |
| --- | --- |
| Cross-replica demo loss | Rust `demo_state_survives_a_different_instance_and_secret` passes mutated cookie state from one independently built app instance to another; Playwright `@claim:demo-replica-continuity` performs 30 reads and a reload. A two-process production-binary probe returned `200` with owner `Sam Rivera` on process two. |
| Spoofable/incomplete limits | Rust tests vary the untrusted first forwarding hop while keeping the trusted final hop fixed. Creation request 6 and write request 31 return `429` with `Retry-After`; the general governor also reaches `429`. Playwright `@claim:rate-limit-policy` covers the production route. |
| Missing claims inventory | All 17 claim tags have count exactly 1. New tests cover cookie attributes/lifetime, continuity, no tracking, body policy, rate policy, security/cache headers, build identity/metrics, and the real CSV workflow. |
| Only a sandbox/dead real action | `/start` now completes a useful real evidence job from import through consent/fallback classification, staff ownership, persistence, and export. It does not claim or attempt regulated patient dispatch without the prerequisites below. |
| Other documented defects | Tests assert `/metrics`, HSTS, Secure cookie, immutable cache, gzip by production probe, JSON error bodies, real 404, skip focus, 44 px footer targets, mutation loading state, and the unpinned Rust base image. |

## Local verification evidence

Environment: Node 22.23.2, npm 10.9.8, rustc/cargo 1.98.0, Playwright 1.58.2.

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 85 packages, 0 vulnerabilities |
| `npm test` | PASS — 6 Vitest, 10 Rust, 21 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/warnings, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` |
| Production bundle | JS 70.56 KB raw / 25.51 KB gzip; CSS 22.12 KB raw / 5.03 KB gzip; fonts 85.96 KB |
| Claims | PASS — 17/17 exact tags, including independent per-claim commands |
| Accessibility | PASS — zero serious/critical axe findings on 6 routes in light and dark; skip focus, keyboard, 44 px targets, reduced motion, and 200% text checked |
| Browser | PASS — desktop and 390 px; no console errors, failed requests, horizontal overflow, or third-party runtime requests |
| HTTP policy | PASS — JSON 400/413, 404 unknown route, HSTS/CSP/nosniff, gzip, immutable asset cache, Secure cookie, `/metrics` Prometheus text |
| Rate limits | PASS — creation 6/6 and write 31/31 return 429 with `Retry-After`; caller-controlled first forwarding hop cannot reset the bucket |
| Replica/restart | PASS — independently constructed apps and two production binary processes preserve the same workspace mutation |
| Lighthouse 12.8.2 local mobile | Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.1 s, LCP 1.4 s, TBT 30 ms, CLS 0 |

Evidence is in `.factory/qa-artifacts/repair/`: desktop/mobile captures, 200% mobile capture, and `lighthouse-local.json`.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
PORT=8080 DIST_DIR=dist target/release/reminder-proof-api
```

Then run `/opt/fleet/lib/verify-url.sh http://127.0.0.1:8080 <evidence-dir>` or any exact command in `.factory/claims.json`.

## Safety boundary and operator actions

The researched full sender cannot be activated honestly in this repair container. Live patient SMS/email/WhatsApp, shared clinic persistence, and subscriptions require all of the following before implementation can pass M2/M3 acceptance:

1. Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Entra SPA and verify the production redirect.
2. Register the subscription product in the Sociobot billing catalog. The required product checkout is not currently registered; no provider checkout was embedded.
3. Supply clinic-approved email/SMS/WhatsApp credentials only server-side, complete BAA/DPA and jurisdiction review, and approve templates/consent policy. No repair test or demo sends a patient message.
4. Provision the planned durable tenant database and complete tenant-isolation, export/deletion, webhook-signature, outbox/idempotency, and provider-failure drills in M2/M3.

Until those actions are complete, `/start` is intentionally an evidence-import and exception tool, not a sender. This is the closest honest useful version permitted by the repository rule for unsafe or impossible scope; the UI, README, Privacy, Terms, claims, and tests state that boundary consistently.

## Deployment

Target: container app `sf-clinic-reminder-proof` at `https://clinic-reminder-proof.sociobot.in`, built with source `Dockerfile`, port 8080. The production app must remain at one replica so in-memory rate allowances have one authoritative bucket; demo state itself is replica-independent.

Live revision, final SHA, and post-deploy checks are recorded after deployment.
