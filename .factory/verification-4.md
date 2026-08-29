# Independent product verification 4 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-4`
Candidate: `26087e3d1b62a948a00e52bb5b060d2a8baded12`
Live URL: https://clinic-reminder-proof.sociobot.in

## Verdict

**FAIL — do not release.** The prior deployment-only concern is resolved: live
`/health` returned this exact SHA. The public demo is functional, private by
default, accessible, and rate-limited. However, this candidate fails the
mandatory claims gate and advertises a subscription which cannot be bought.

## First-read gate — PASS

Fresh cold browser read, before using the product:

- **What:** “See every reminder outcome.”
- **For whom:** “For independent clinics that need delivery proof and a clear
  next step when reminders fail.”
- **First action:** visible one-click **“Try it with sample data”**, adjacent
  to “Opens a sample clinic. Nothing touches real clinic data.”

It opened the isolated sample clinic. This satisfies the plain-words and
one-click-demo requirements.

## Release blockers

### Critical — required claims test fails

The exact required command below fails from the clean checkout after `npm ci`:

```text
npm run test:e2e -- --grep @claim:rate-limit-policy
Expected: 429
Received: 200
```

The test changes the *first* `X-Forwarded-For` hop for every one of its six
requests (`192.0.2.0` through `.5`). That represents six different clients,
so its expected 429 is impossible under the documented per-client policy. The
production implementation is not the observed defect: retaining one first hop
(`198.18.254.17`) and varying only a later proxy hop returned
`200, 200, 200, 200, 200, 429, 429`; the sixth response was JSON
`{"code":"rate_limited",...}` with `Retry-After: 3599`.

Regardless of the cause, any failing declared claim is release-blocking. Fix
the test to keep the first hop stable, prove the `429`/`Retry-After` result,
and run the exact command green.

### Critical — advertised $79/month plan is unavailable

The landing and Terms publicly advertise the Clinic plan. A fresh production
request to the app's documented Sociobot checkout boundary returned:

```text
GET https://api.sociobot.in/api/v1/products/clinic-reminder-proof/checkout?... 
HTTP/2 404
{"error":"enabled factory product","status":404}
```

Thus a clinic cannot purchase the promised subscription. This repeats the
unresolved operator dependency recorded in the previous handoff. Register and
enable the recurring product in the production Sociobot catalog, then perform
a real pilot checkout/return/verification cycle.

### High — claims manifest is structurally non-compliant

`npm test` fails immediately in `tests/contracts.test.ts` because the two
claims below do not contain their required `@claim:<id>` test tag:

- `managed-provider-fallback-receipt`
- `managed-billing-return`

Their commands name the same untagged Rust fixture test instead. The exact
commands happen to pass, but the claims contract requires exactly one tagged,
observable test per claim. Add distinct tagged tests/commands and retain the
fixture assertions.

### High — real clinic storage remains explicitly not ready

The current README and `.factory/handoff.md` say the non-root container cannot
prepare/mount the durable Azure Files volume and instruct operators not to
accept real clinic records until the storage work is complete. The public UI
still offers “Start for real.” Durable shared storage, backup/restore, and a
restart/replica proof are required before accepting sensitive clinic data.

## Claims matrix (clean checkout demo entry point)

`npm ci` installed 87 packages with 0 vulnerabilities. Every command from
`.factory/claims.json` was invoked before general QA. The individual failing
test has a screenshot/video/trace in
`test-results/m1-claims--claim-rate-limi-0edee-ss-and-returns-Retry-After--chromium/`.

| Claim ID | Exact command result | Contract result |
| --- | --- | --- |
| demo-isolation | PASS | PASS |
| sample-outcome-coverage | PASS | PASS |
| consent-channel-guard | PASS | PASS |
| fallback-order | PASS | PASS |
| delivery-timeline | PASS | PASS |
| exception-ownership | PASS | PASS |
| demo-reset | PASS | PASS |
| minimal-reminder-content | PASS | PASS |
| public-price | PASS | PASS |
| demo-cookie-lifetime | PASS | PASS |
| demo-replica-continuity | PASS | PASS |
| no-tracking | PASS | PASS |
| request-protection | PASS | PASS |
| rate-limit-policy | **FAIL** — expected 429, received 200 | **FAIL** |
| security-headers | PASS | PASS |
| build-identity | PASS | PASS |
| managed-auth-storage | PASS (Rust + tagged browser test) | PASS |
| managed-provider-fallback-receipt | PASS (Rust fixture) | **FAIL** — no `@claim:managed-provider-fallback-receipt` tag |
| managed-billing-return | PASS (same Rust fixture) | **FAIL** — no `@claim:managed-billing-return` tag |

## Local gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages, 0 vulnerabilities |
| `npm test` | **FAIL** — Vitest claim-contract failure above; it therefore does not reach later scripts |
| `npm run test:api` | PASS — 21 Rust tests |
| `npm run test:e2e` | **FAIL** — 1 failing test: `@claim:rate-limit-policy`; all other 21 tests completed without a failure artifact |
| `npm run check` | PASS — Svelte 0 errors/warnings, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` |
| Release runtime with only `PORT=18080` | PASS — `/health` returned `{"status":"ok","build_sha":"dev"}`; generated default data key/store reported in structured startup log |
| Local health concurrency | PASS — 100 concurrent `/health` requests: 100 × 200 |

The public production entry bundle is 80,084 bytes raw / 27,900 gzip; CSS is
24,505 / 5,310 gzip. The 271,994-byte MSAL chunk is lazy and not fetched on
the public landing. These are within the stated first-load budgets.

## Live functional, privacy, backend, and security evidence

- `/health` returned `{"status":"ok","build_sha":"26087e3d1b62a948a00e52bb5b060d2a8baded12"}` and `/metrics` returned Prometheus text.
- Public landing loaded only same-origin document, JS, CSS, and self-hosted
  font. A complete fresh demo flow (advance, assign Sofia R. to Sam Rivera,
  resolve, reload, undo, reset) made 20 requests, all to the same origin, with
  no console errors, page errors, or failed requests.
- The demo cookie is `rp_demo`, `HttpOnly`, `Secure`, `SameSite=Lax`, scoped
  to `/api/v1/demo`, and expires in 24 hours. Reset restored the original
  seed once the completion notice was awaited.
- Browser responses include CSP with `frame-ancestors 'none'`, HSTS,
  `X-Content-Type-Options: nosniff`, strict-origin referrer policy,
  permissions policy, and COOP. Production hashed assets use one-year
  immutable caching.
- Unauthenticated `/api/v1/clinic` and export returned `401` with
  `WWW-Authenticate: Bearer`; public auth configuration names only the
  required `sociobotcustomers.ciamlogin.com` tenant.
- Live rate-limit observation: demo creation permits **5 requests per client
  per hour**, then returns 429 plus `Retry-After`. The test must preserve the
  first forwarded hop to exercise this behavior.

## Accessibility, responsive, and performance evidence

- Live desktop and 390 px mobile sample flows were exercised. First Tab
  focused the visible “Skip to main content” link; keyboard actions and the
  44 px controls worked. Reduced-motion mode was enabled during the mobile
  check.
- Playwright axe found **zero serious/critical** violations on the live
  landing and demo at desktop and 390 px. No console/page errors were seen.
- Lighthouse mobile (Chromium with `--disable-dev-shm-usage`): Performance
  **100**, Accessibility **100**, LCP **1.4 s**, TBT **50 ms**, CLS **0**,
  total network **58 KiB**.

## Required path to PASS

1. Correct `@claim:rate-limit-policy` to use one stable first
   `X-Forwarded-For` hop and prove its 429 response; make the full suite green.
2. Give provider-fallback and billing-return their own `@claim:`-tagged,
   fixture-backed observable tests and make `npm test` pass.
3. Register the production recurring Sociobot Clinic plan and verify a pilot
   checkout, return, entitlement activation, and invalid/revoked result.
4. Do not accept clinic records until durable non-root-writable storage,
   backup/restore, and restart/replica persistence are proven.

No product code was modified during this verification.
