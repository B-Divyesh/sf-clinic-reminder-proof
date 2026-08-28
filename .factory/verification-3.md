# Independent product verification 3 — FAIL

Date: 2026-08-28 UTC

Work order: `clinic-reminder-proof-verify-3`

Candidate: `bc5592916143ff182424878a6cf60ef057d7007e`

Live URL: `https://clinic-reminder-proof.sociobot.in`

## Verdict

**FAIL — do not release.**

The prior deployment-only concern is not current: `/health` reports the exact candidate SHA, and the live JS/CSS hashes exactly match the clean local build. The product still fails the acceptance contract because its advertised subscription checkout is not enabled and its mandatory per-client limits are not enforced across live replicas.

## Release blockers

### Critical — the advertised Clinic subscription cannot be purchased

The landing page and terms advertise the `$79 per location each month` Clinic plan, `/start` says subscription checkout opens after setup, and the managed-workflow claim includes Sociobot subscription checkout. The backend constructs this production URL:

`https://api.sociobot.in/api/v1/products/clinic-reminder-proof/checkout?tier=clinic&return_url=...&organization_id=...`

A fresh production GET to that exact product checkout boundary returned:

```text
HTTP/2 404
{"error":"enabled factory product","status":404}
```

The candidate handoff likewise says the monthly product still needs operator registration. A stranger can reach the correct Sociobot CIAM sign-in page, but cannot complete the promised payment path. This violates the real-product and under-five-minute purchase acceptance criteria.

Required fix: register and enable the recurring `clinic-reminder-proof` product in the Sociobot billing catalog, then add a claim test that completes a test-mode checkout/return/verification cycle rather than checking only that a route or URL exists.

### High — rate limits are replica-local and can be exceeded

Both limiters are process memory: `DemoStore` keeps its 5/hour creation timestamps in an in-memory `HashMap`, and `tower_governor` keeps the general 20 requests/second, burst-40 state inside each app instance. Production serves multiple instances.

Fresh live evidence from one client, varying only the untrusted first `X-Forwarded-For` hop, was:

```text
creation probe: 200, 200, 200, 429 Retry-After: 3484, 200, 200, 429 Retry-After: 3592
general probe: 120 concurrent /metrics requests in 593 ms -> 90 x 200, 30 x 429
```

All 30 general-limit rejections and both creation-limit rejections included `Retry-After`, but a successful response followed a 429 immediately. There is therefore no stable service-level allowance for one client. The documented strict allowance is 5 creations/hour **per replica**, not per client; the effective live allowance scales with replica count. The general burst likewise exceeded 40 materially.

Required fix: enforce limits in a shared store or at ingress, or use a deployment topology that guarantees one limiter state per client. Add a production-topology regression that alternates requests across at least two instances and proves every request after the allowance returns 429 with `Retry-After`.

### High — the broad managed-workflow claim does not prove its observable outcome

The exact `@claim:managed-clinic-workflow` browser test passes, but it checks copy, CIAM configuration, unauthenticated 401 responses, and route presence. Its Rust command proves tenant-scoped storage, signed intake, consent blocking, and signature helper behavior, but never performs an accepted provider dispatch/receipt/fallback through a fixture HTTP provider and never completes billing checkout/return.

This is why the claim command passes while the live checkout is 404. The claims contract requires the promised result, not the presence of controls or endpoints. Split the broad claim into independently observable auth, durable workspace, connector, provider dispatch/receipt/fallback, exception, export/delete, and billing claims with fixture-backed end-to-end tests.

## Other findings

### High — production durability and backups are not acceptance-ready

The implementation persists clinic data to SQLite under `DATA_DIR`, but the candidate handoff still says persistent `/data` storage must be confirmed before accepting real records. The repository has no backup/restore procedure or test, and `/health` does not expose a storage-readiness boundary. A live signed-in tenant was not created because no verifier identity or clinic-approved provider credentials were supplied, so cross-replica managed persistence could not be proven from the public interface.

The product currently invites “Start for real.” Confirm shared durable storage and backups, document restore, and exercise tenant persistence across restart/replica before accepting clinic data.

### Medium — the generated clinic encryption key is world-readable on the host

Starting the release binary with only `PORT` correctly generated its key and SQLite store, but both were mode `0644`:

```text
644 root:root target/reminder-proof-data/clinic-data.key
644 root:root target/reminder-proof-data/clinic-data.sqlite3
```

Create the key and database with owner-only permissions (`0600`) and keep the containing directory private. No key or provider secret was observed in browser responses.

## Mandatory first gates

### Claims manifest — installed clean-checkout PASS, coverage defect above

`.factory/claims.json` exists with 17 entries. Per the work order, every listed command was invoked before other repository inspection. That pre-install invocation could not load the declared `@playwright/test` package. After the required clean `npm ci` (87 packages, 0 vulnerabilities), every exact command was rerun independently and passed: 17/17 entries, including 5/5 filtered Rust tests for the combined managed claim. Every claim tag occurs exactly once.

Passed IDs: `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline`, `exception-ownership`, `demo-reset`, `minimal-reminder-content`, `public-price`, `demo-cookie-lifetime`, `demo-replica-continuity`, `no-tracking`, `request-protection`, `rate-limit-policy`, `security-headers`, `build-identity`, and `managed-clinic-workflow`.

The command results are green, but the managed claim is inadequately scoped and contradicted by the live checkout evidence described above.

### Cold first-read — PASS

The live first screen answers all three required questions in plain words:

- What: “See every reminder outcome.”
- For whom: “For independent clinics that need delivery proof and a clear next step when reminders fail.”
- First click: “Try it with sample data,” next to “Opens a sample clinic. Nothing touches real clinic data.”

The action opens `/demo` in one click. Evidence: `.factory/qa-artifacts/live-cold-desktop.png`.

## Local build and automated gates

Environment: Node 22.23.2, npm 10.9.8, rustc/cargo 1.98.0, Playwright 1.58.2.

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages, 0 vulnerabilities |
| Every exact claim command | PASS — 17/17 after install |
| `npm test` | PASS — 6 Vitest, 18 Rust, 22 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; rustfmt; clippy `-D warnings` |
| `npm run build` | PASS — `dist/` and release API binary produced |
| Default runtime | PASS — starts with only `PORT`, generates/persists config, structured JSON startup logs |
| Local health concurrency | PASS — 100/100 returned 200 |
| Docker build | Not run — no Docker engine in the verifier container; static Dockerfile inspection passes the required shape |

Production output: main JS 80,084 bytes raw / 27,485 gzip; CSS 24,505 / 5,331; landing font 25,224 bytes. The sign-in-only MSAL chunk is lazy (271,994 raw / 67,360 gzip). The public first load is within budget.

## Live deployment and functional evidence

- `/health` returned `{"status":"ok","build_sha":"bc5592916143ff182424878a6cf60ef057d7007e"}`. One hundred concurrent health requests all returned 200 with that SHA.
- Live `index-fPRgf9TY.js` and `index-D1-NjmEw.css` SHA-256 hashes exactly match the local production build.
- `/metrics` returns Prometheus text. `/`, `/demo`, `/start`, `/app`, `/privacy`, and `/terms` return 200; an unknown route returns the styled page with HTTP 404.
- In one live demo workspace, “Advance due reminders” changed delivery evidence from 1 to 3; Jordan L. showed WhatsApp rejection then email fallback. Sofia R. was assigned to Sam Rivera, resolved, reloaded with owner/resolution intact, and safely undone.
- Invalid owner returned structured `422 owner_invalid`; malformed JSON returned structured `400 json_invalid`. A prior 17 KB body check is covered by the passing claim and returns 413.
- The demo cookie is `HttpOnly`, `Secure`, `SameSite=Lax`, scoped to `/api/v1/demo`, and expires in 24 hours.
- Normal landing/demo use contacted only `https://clinic-reminder-proof.sociobot.in`. No analytics, CDN font, provider, billing, or AI request occurred. Valid routes produced no console/page errors; the deliberate 400/422 probes produced only expected failed-resource console entries.
- Browser headers include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP. Hashed assets use `public, max-age=31536000, immutable`.
- The public auth configuration is the exact Sociobot tenant/client. “Sign in with Microsoft” reached the registered PKCE authorization page at `sociobotcustomers.ciamlogin.com` with redirect URI `https://clinic-reminder-proof.sociobot.in/auth/callback`; there was no redirect-registration error.

## Accessibility, responsive behavior, and performance

- Axe found zero violations (not merely zero serious/critical) on `/`, `/demo`, `/start`, `/app`, `/privacy`, `/terms`, and the 404 view.
- Each tested route has `lang=en`, one h1, one main landmark, and a route-specific title. First Tab shows a 3 px `#005fcc` skip-link focus ring; Enter focuses `main`.
- At 390 px with 200% root text, `/start` had 390 px document width, no horizontal overflow, no visible control under 44 px, and no console errors.
- Reduced-motion media matched and all durations were reduced to `0.00001s`.
- Lighthouse 12.8.2 mobile: Performance 92, Accessibility 100, Best Practices 100, SEO 100; FCP/LCP 1.2 s, CLS 0, TBT 340 ms.
- PWA/offline-install checks and package/CLI consumer checks are not applicable; this is not a PWA, library, or CLI.

Screenshots: `.factory/qa-artifacts/verification-3-live-demo-desktop.png`, `.factory/qa-artifacts/verification-3-live-start-mobile-200.png`, and `.factory/qa-artifacts/verification-3-live-auth.png`.

## Required path to PASS

1. Enable the recurring Sociobot product and prove the full test checkout/return/verification flow.
2. Make rate-limit state service-wide and rerun a multi-replica single-client probe.
3. Replace the broad managed-workflow presence test with fixture-backed observable outcome claims, including accepted dispatch, signed receipt, terminal fallback, checkout, export, and deletion.
4. Confirm live `/data` persistence and backups across restart/replica, then restrict key/database file modes.

