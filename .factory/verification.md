# Independent product verification — FAIL

Date: 2026-08-28 UTC

Work order: `clinic-reminder-proof-verify-1`

Candidate: `6e4cbb77f20f9668b9d0f27dc9e257eb790e6fe1`

Live URL: `https://clinic-reminder-proof.sociobot.in`

Verdict: **FAIL — do not release**

This was a fresh independent verification. No product code was changed. The live health response reports the exact candidate SHA, and the live JS/CSS bytes match the clean local production build. The previous builder's deployment-only failure is no longer current: the candidate is deployed, but the live demo is not reliable across requests.

## Release blockers

### Critical — the live demo loses its workspace between requests

The backend generates a cookie-signing key at boot and keeps demo workspaces only in process memory. The live service routes requests across instances that do not share that key or state.

- After creating one live workspace, 24 sequential `GET /api/v1/demo/state` calls with the same cookie returned 14 × `200` and 10 × `401 demo_cookie_invalid`.
- In a browser, a normal detail → Back → assign flow produced `401` on state, auto-created another workspace, then returned `401` on assignment with “Start a new sample clinic.”
- User changes can therefore disappear, and the required smallest demo journey cannot be completed reliably on the deployment.
- The same end-to-end flow passes against one local server process, which explains why the checked-in tests miss this production topology failure.

Required fix: use a shared signing secret and a shared/affinitized TTL workspace store, or guarantee single-replica routing for the demo and verify restart behavior.

### Critical — this is only a simulated M1 sandbox, not the brief's smallest useful product

The acceptance contract requires an EMR/calendar connector, consent-aware template sending, real approved-channel fallbacks, delivery/response ingestion, and a staff exception queue. The candidate has no account, connector, provider integration, durable clinic persistence, real dispatch, subscription flow, or usable “Start for real” path. The page states this limitation honestly, but a clinic cannot perform the real job-to-be-done. This fails the repository Definition of Done for an end-to-end product rather than a demo.

### High — mandatory rate-limit responses are incomplete and the client key is spoofable

- Documented workspace creation allowance: first 5 requests per hour returned `200`; requests 6 and 7 returned `429` **without `Retry-After`**.
- Documented demo-write allowance: first 30 writes per minute returned `200`; writes 31 and 32 returned `429` **without `Retry-After`**.
- General API governor: burst 40. In a 60-request concurrent probe after workspace creation, 39 reads returned `200` and 21 returned `429`; those governor rejections did include `Retry-After: 19`.
- The live ingress preserves a caller-supplied first `X-Forwarded-For` value. Varying it selects a new limiter bucket, so an external client can bypass the per-IP allowances.

The backend contract requires every limited endpoint to return `429` with `Retry-After`, keyed to the ingress-established client address. This is independently release-blocking.

### High — claims inventory is incomplete

All nine declared claims pass, but the landing page, Privacy page, and README contain visitor-reliant claims missing from `.factory/claims.json`. Examples include signed/HttpOnly cookie isolation, random workspace creation, expiry within 24 hours, no tracking script, request/body limits, security headers, and the health endpoint. Some have unit coverage, but the claims contract requires each claim to have exactly one demo-entry test in `claims.json`. An unlisted claim is an explicit release failure.

## Other defects

### High

- `/metrics` is not implemented. The route returns the SPA HTML with `200 text/html`, so the service lacks the mandatory operational metrics endpoint.
- Production responses have no `Strict-Transport-Security`. The demo cookie is `HttpOnly; SameSite=Lax; Path=/api/v1/demo; Max-Age=86400` but lacks `Secure`.
- The Dockerfile pins `FROM rust:1.98.0-bookworm`; the backend contract explicitly requires `rust:1-slim` or `rust:1-alpine` and forbids a pinned minor. Docker was unavailable in this verifier container, so an image build could not be repeated.

### Medium

- Unknown browser routes return `200` with the styled not-found page rather than an HTTP `404`.
- Hashed JS/CSS/font assets have no `Cache-Control` and no content encoding. Lighthouse flags text compression (estimated 53 KiB) and cache lifetimes (estimated 105 KiB).
- The skip link is first in keyboard order and visibly focused, but Enter leaves focus on `<body>` instead of moving focus to `<main>`.
- At 390 px, footer links are only 16 px high; the header Demo link measures 43 × 44 px. These miss the 44 × 44 px touch-target contract.
- An immediate owner change after browser Back can race the demo-state reload because the select remains enabled while `loading` is true. The saved state may be replaced visibly by the stale read.

### Low

- Invalid JSON and over-limit bodies return framework text such as “Failed to parse…” rather than the API's structured, action-oriented problem shape.

## Mandatory claims gate

The clone started clean at the requested commit. The first pre-install invocation could not load `@playwright/test`; after the required `npm ci`, every exact command from `.factory/claims.json` executed against `/?demo=1` and passed.

| Claim | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS |

Each claim tag occurs exactly once in `tests/e2e/m1-claims.spec.ts`.

## First-read gate

**PASS.** Cold desktop and 390 px loads immediately say:

- What: “See every reminder outcome.”
- For whom: independent clinics that need delivery proof and a next step after a failed reminder.
- First action: “Try it with sample data,” with adjacent text explaining it opens a sample clinic.

The action is one click from the first screen. Screenshots: `qa-artifacts/live-first-read-desktop.png` and `qa-artifacts/live-first-read-mobile.png`.

## Build and automated gates

Environment: Node `22.23.2`, npm `10.9.8`, rustc/cargo `1.98.0`.

| Check | Result |
| --- | --- |
| `npm ci` | PASS; 85 packages, 0 vulnerabilities |
| `npm test` | PASS; 6 Vitest + 6 Rust + 12 Playwright |
| `npm run check` | PASS; Svelte 0 errors/warnings, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS; `dist/` and release API binary produced |
| Production bundle | PASS budget; JS 62.39 KB raw / 23.10 KB gzip, CSS 19.40 KB raw / 4.69 KB gzip, fonts 85.96 KB total |
| Default runtime | PASS; release binary starts with only `PORT`, generates/persists its secret, emits JSON startup log |
| Local health concurrency | PASS; 100 concurrent requests returned 100 × `200` |
| Live health concurrency | PASS; 100 concurrent requests returned 100 × `200`, all with candidate SHA |

## Independent functional coverage

Against a single local production-mode server:

- Advanced all due reminders: Due 4, Delivered 3, Exceptions 1.
- Verified WhatsApp rejection followed by consented email fallback.
- Assigned Sofia R. to Sam Rivera, resolved as Called patient, reloaded to prove persistence, undid, and reset.
- Reset changed the random workspace ID and restored original seed states.
- A tampered signed cookie recovered by creating a new isolated workspace.
- Two browser workspaces had different IDs; advancing A left B at one delivered reminder.
- Invalid owner returned `422`; resolve-without-owner `409`; unknown reminder `404`; malformed JSON `400`; 17 KB body `413`; missing cookie `401`.
- All browser requests in the complete local flow were same-origin and produced no console errors.

The live equivalent failed because of the critical cross-instance session defect above.

## Accessibility, responsive behavior, and performance

- Axe: zero violations (including zero serious/critical) on `/`, `/privacy`, `/terms`, the not-found view, and `/demo`, in light and dark schemes.
- Semantics: `lang=en`, one h1, one main landmark, route-specific titles, ordered headings, labels, and visible 3 px focus rings.
- Mobile: no horizontal overflow at normal 390 px layout; body text is 16 px. Demo and landing captures are under `qa-artifacts/`.
- Reduced motion: media query matched and animations were removed/reduced to near-instant durations.
- Lighthouse 12.8.2 mobile: Performance 99, Accessibility 100, Best Practices 100, SEO 100; FCP/LCP 1.6 s, TBT 30 ms, CLS 0. Report: `qa-artifacts/lighthouse-live.json`.
- Initial live transfer measured 109,033 bytes: HTML 806 bytes, JS 62,399, CSS 19,404, one font 25,224.

Manual accessibility defects are listed above; axe does not detect touch-target size or the ineffective focus handoff of the skip link.

## Privacy and deployment identity

- Browser request logging on cold landing and one-click demo entry showed only `https://clinic-reminder-proof.sociobot.in` requests; no analytics, CDN fonts, providers, billing, or AI endpoints were contacted.
- CSP, `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`, and COOP are present. HSTS, `Secure` cookie, caching, and compression are absent.
- `/health` returns `{"status":"ok","build_sha":"6e4cbb77f20f9668b9d0f27dc9e257eb790e6fe1"}`.
- Live JS and CSS SHA-256 hashes exactly match the locally built candidate artifacts.
- PWA/service-worker checks are not applicable: the candidate does not claim or implement a PWA.
- CIAM checks are not applicable to an implemented sign-in flow: there is no sign-in. The absence of real onboarding is part of the product-scope blocker.

## Reproduction priorities

1. `curl` one workspace cookie, then repeatedly call `/api/v1/demo/state`; observe intermittent `401 demo_cookie_invalid` live.
2. Send six workspace-creation requests as one client; request six returns `429` without `Retry-After`.
3. Open `/metrics`; observe `200 text/html` containing the SPA shell.
4. Open any missing path and inspect the network status; it is `200`, not `404`.
5. Inspect the live `Set-Cookie`; `Secure` is absent.
