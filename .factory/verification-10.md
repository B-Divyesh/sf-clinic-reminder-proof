# Independent product verification 10 — PASS

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-10`
Candidate: `741bba6617bbf5673e8b2b986a7f435496e6ed24`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**PASS — candidate accepted.** The earlier deployment-only concern is not reproducible: live `/health` returns the exact requested candidate identity:

```json
{"status":"ok","build_sha":"741bba6617bbf5673e8b2b986a7f435496e6ed24"}
```

No release-blocking defects were found.

## First-read result

Cold desktop production load answered all three required questions in plain words:

- **What it does:** “See every reminder outcome.”
- **For whom:** independent clinics that need delivery proof and a clear next step after a reminder fails.
- **What to do first:** the visible one-click **“Try it with sample data”** action, immediately explained as opening a sample clinic without touching real clinic data.

The action opened the isolated `/demo` sandbox. It has the persistent “Demo — sample data, nothing is saved to your clinic” banner, Reset demo, and Start for real actions.

## Claims gate — 31/31 PASS

`.factory/claims.json` exists. From the clean candidate checkout I ran `npm ci`, then executed every manifest `test` command exactly as declared, one at a time, using the shipped demo entry point. All passed; individual command output was retained during verification as `/tmp/claim-<id>.log`.

| Claim | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `sample-exception-visibility` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS |
| `demo-cookie-lifetime` | PASS |
| `demo-replica-continuity` | PASS |
| `no-tracking` | PASS |
| `explicit-theme-choice` | PASS |
| `request-protection` | PASS |
| `rate-limit-policy` | PASS |
| `security-headers` | PASS |
| `build-identity` | PASS |
| `managed-auth-storage` | PASS |
| `signed-calendar-intake` | PASS |
| `approved-whatsapp-dispatch` | PASS |
| `twilio-receipt-verification` | PASS |
| `resend-receipt-verification` | PASS |
| `managed-secret-encryption` | PASS |
| `managed-data-minimisation` | PASS |
| `no-marketing-campaigns` | PASS |
| `signed-in-export-delete` | PASS |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS |

## Local quality gates

- `npm ci`: PASS — 87 locked packages installed; audit reported 0 vulnerabilities.
- `npm test`: PASS — 8 Vitest tests, 33 Rust tests, and 39 Chromium browser tests.
- `npm run check`: PASS — Svelte check, `cargo fmt --check`, and Clippy with warnings denied.
- `npm run build`: PASS — emitted `dist/` and `target/release/reminder-proof-api` (13,487,304 bytes).
- Release executable: PASS — started on `PORT=4811` with no configuration supplied, served `/health`, and logged generated local durable-key configuration without exposing a secret. A 20-way, 100-request `/health` smoke returned 100 × 200.
- Container build: not run because the verification container has no `docker` executable. This is not a product failure; the independently deployed service above is serving the exact candidate SHA.

## Independent live functional checks

- With a fresh demo client, **Advance due reminders** produced 4 due reminders, 3 delivery-evidence outcomes, and 1 staff-owned exception. Jordan L.’s evidence recorded `TEMPLATE_REJECTED` followed by the simulated email fallback.
- Sofia R.’s exception could be assigned to Sam Rivera, resolved as “Called patient,” persisted through reload, undone, and reset to the canonical seed (4 due / 1 delivered / 1 exception).
- A malformed public write returned `415` JSON with `code: "content_type_invalid"`, a clear recovery message, and a matching UUID `X-Request-Id`.
- Public routes `/`, `/demo`, `/privacy`, `/terms`, `/start`, `/404`, `/robots.txt`, and `/sitemap.xml` all returned 200. An unknown route returned HTTP 404.
- Live `/metrics` returned Prometheus text. The exact Sociobot Entra External ID authority and shared client ID are returned by `/api/v1/auth/config`; no other sign-in authority was observed.

## Privacy, security, accessibility, and performance

- A cold landing request log on desktop and 390 px mobile contained only same-origin HTML, JS, CSS, and self-hosted font requests. A complete demo interaction request log remained same-origin; no tracking, provider, billing, or third-party runtime request appeared.
- No page errors or console errors occurred during the cold browser and demo checks.
- Axe on both desktop and 390 px mobile found **0 serious and 0 critical** violations. Mobile had no horizontal overflow (`scrollWidth = 390`).
- Keyboard traversal reached the skip link, header navigation, theme selector, primary demo action, and footer in order. Every inspected focus target had the designed `rgb(0, 95, 204)` 3 px outline; controls were at least 44 px high.
- The shipped reduced-motion rule shortens all animation/transition duration to `.01ms`; the browser suite’s reduced-motion coverage passed.
- Browser responses include CSP with `frame-ancestors 'none'`, HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, permissions policy, and COOP. The hashed JS has `Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.
- Production initial JS was 82,382 bytes raw / 28,129 bytes gzip; CSS was 25,829 bytes raw / 5,544 bytes gzip, within the stated budgets.

## Rate limiting and backend boundaries

Live demo creation was independently exercised with one stable first `X-Forwarded-For` hop while later hops varied: five creates returned 200; request six returned `429` with `Retry-After: 3599` and structured JSON. Thus the observed documented allowance is **five demo-workspace creations per client per hour**. The API also publishes its general governor limit (`X-RateLimit-Limit: 40`).

The passed claims and Rust tests cover encrypted durable data, tenant ownership, signed calendar intake, callback signature/replay handling, consent gate/fallback order, minimised export/delete, body limits, and storage recovery. No real patient data, clinic credentials, provider dispatch, billing purchase, or customer deletion was used in this verification.

## Defects by severity

| Severity | Findings |
| --- | --- |
| Critical | None |
| High | None |
| Medium | None |
| Low | None |

## Reproduce

```sh
npm ci
npm test
npm run check
npm run build
```

Use <https://clinic-reminder-proof.sociobot.in/?demo=1> for the one-click sandbox.
