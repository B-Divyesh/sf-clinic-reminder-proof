# Repair 4 handoff — release blockers repaired

Date: 2026-08-29 UTC  
Work order: `clinic-reminder-proof-repair-4`  
Base verifier report: `e59bd22495b3efe9621c5875fb5131e45c4aa58f`  
Rejected candidate: `26087e3d1b62a948a00e52bb5b060d2a8baded12`

## What changed

- Reproduced the exact declared `@claim:rate-limit-policy` failure first
  (`Expected: 429`, `Received: 200`). The browser regression now keeps one
  first `X-Forwarded-For` hop, varies only the proxy hop, asserts creates one
  through five are 200, and asserts create six is structured 429 with a
  positive `Retry-After`.
- Split the managed provider and billing fixture into independent observable
  Rust scenarios. Each claim now has its own exact Playwright
  `@claim:managed-provider-fallback-receipt` or
  `@claim:managed-billing-return` test and manifest command.
- Preserved the public `$79 per location each month` price and the checkout
  path `https://api.sociobot.in/api/v1/products/clinic-reminder-proof/checkout`.
  Catalog registration is controller-owned and was not changed here.
- Made clinic persistence deployable by a non-root process without an init
  container. Production uses one replica and two direct ReadWrite Azure Files
  mounts: `/data` for the encrypted SQLite store/key and `/backups` for a
  separate online SQLite backup/key pair.
- Every successful workspace save or deletion takes a SQLite online backup
  while holding the database mutex, atomically replaces the matching latest
  files, and retains daily recovery pairs for 30 days.
  `managed_backup_pair_restores_after_database_loss` restores
  both files into a clean directory and reads the original tenant record.

## Verification evidence

- `npm ci`: PASS, 87 packages, zero vulnerabilities.
- Exact pre-fix reproduction: `npm run test:e2e -- --grep
  @claim:rate-limit-policy` failed 429 vs 200 as reported.
- Exact repaired claims: rate limit, managed provider fallback/receipt, and
  managed billing return all PASS independently and together.
- `npm test`: PASS — 6 Vitest contracts, 23 Rust API tests, 24 Chromium tests.
- `npm run check`: PASS — Svelte 0 errors/warnings, rustfmt, clippy
  `-D warnings`.
- `npm run build`: PASS — `dist/` and the release API binary produced. Public
  entry JS is 80,084 bytes raw / 27,900 gzip; CSS is 24,505 / 5,310 gzip.
- Browser suite: PASS at desktop and 390 px/200% text, keyboard-only and
  reduced-motion paths, deep links/back navigation, offline read behavior,
  local-link crawl, console check, same-origin privacy request log, and zero
  serious/critical axe findings across both color schemes.
- Only-`PORT` release runtime: PASS on port 18080; `/health` returned
  `{"status":"ok","build_sha":"dev"}` and generated owner-only local data
  and backup directories without other configuration.

## Production deployment and live checks

The artifact remains a single container on port 8080. The two Azure Files
storage bindings are ReadWrite and the checked-in deployment contract fixes
`minReplicas=1` and `maxReplicas=1`. Live deployment evidence is recorded in
the final section below after the release revision is activated.

## Needs operator action

Recurring product registration is handled separately by the controller, per
this repair work order. The code and fixture retain and prove the required
Sociobot checkout/return contract. Clinic-owned provider credentials,
approved message templates, consent policy, BAA/DPA review, and daily Azure
Files snapshot retention remain operational responsibilities.

---

# Independent verification 4 handoff — FAIL

Date: 2026-08-29 UTC
Candidate and deployed build: `26087e3d1b62a948a00e52bb5b060d2a8baded12`
Live URL: https://clinic-reminder-proof.sociobot.in

**Status: FAIL — do not release.** Fresh `/health` confirms the live service
is this exact candidate. The public first-read/demo gate, public demo flow,
privacy request log, headers, 390 px keyboard flow, axe, bundle budget, and
stable-client demo rate limiting all pass. The candidate nevertheless has
release-blocking failures:

1. The mandatory exact `@claim:rate-limit-policy` command fails: it varies
   the first forwarded client hop on every request and receives 200 rather
   than the test's expected 429. A proper stable-client live probe shows the
   intended allowance of five creations then `429` with `Retry-After: 3599`.
2. `npm test` fails its own claims contract because
   `managed-provider-fallback-receipt` and `managed-billing-return` do not
   have required `@claim:<id>` tests.
3. The public `$79/location/month` Sociobot checkout is unavailable: the
   documented production product endpoint returns HTTP 404
   `{"error":"enabled factory product","status":404}`.
4. Durable managed-clinic storage and backups remain explicitly unready; do
   not accept real clinic records.

Verification details, every claim result, local output, live evidence, and
required fixes are in [verification-4.md](verification-4.md). Local verifier
results: `npm run test:api`, `npm run check`, and `npm run build` passed;
`npm test` and the full Playwright suite failed only on the documented claims
gate. No product source code was changed.

---

# Reminder Proof repair 3 handoff

Status: **not release-ready — two external acceptance items remain**

Date: 2026-08-29 UTC

Repair commits: `20593be`, `66af0ea`, `51226a9`, `8ac5e5b`

Deployed build: `8ac5e5b635d353d58ca19bddb9176dc04394400c`

## Repaired and verified

- The demo-creation allowance now keys on the first ingress client hop, not a proxy hop. Production is pinned to one replica. A live six-request probe returned `200, 200, 200, 200, 200, 429`; the sixth response included `Retry-After: 3599`.
- Generated local clinic directories, data keys, and SQLite files are owner-only (`0700`, `0600`, `0600`), with a regression test. Azure Files rejects POSIX chmod; the code records that managed-share limitation rather than crashing.
- The broad managed-workflow claim is split into auth/storage, provider fallback/receipt, and billing-return claims. A fixture HTTP provider rejects SMS, accepts email, persists both outcomes, accepts a receipt, and a fixture Sociobot checkout/verify cycle activates the subscription.
- SQLite now selects a rollback journal rather than WAL for SMB compatibility. A single-replica Azure Files share and environment storage binding were provisioned, but its mount cannot be prepared by the non-root runtime image; the production revision deliberately remains unmounted and starts normally.
- Clean local checks completed: `npm ci`, `npm run check`, `npm test`, `npm run build`, all exact claim commands, and `cargo test --manifest-path services/api/Cargo.toml`.
- Live `/health` returned the deployed SHA above. The deployed Container App has `minReplicas=1` and `maxReplicas=1`.

## Still blocking release

1. Sociobot’s production catalog does not contain `clinic-reminder-proof`: its checkout remains HTTP 404. The live Dodo credential available to this worker authenticates only against test mode, and the factory catalog schema currently holds one-time product records; no recurring product could be honestly registered from this work order. The application now fails its same-origin checkout route explicitly when the catalog endpoint is unavailable rather than handing a clinic a broken URL.
2. Durable managed clinic storage and backups are not release-ready. The Azure Files share is provisioned, but it cannot be initialized by the non-root container and SQLite cannot be accepted there until a compatible volume/database path is supplied. The app must not accept real clinic records while this remains unresolved.
3. The live general `/metrics` burst still needs an ingress-safe test with the actual factory forwarding chain; a 60-request synthetic header burst was all 200 despite the in-process governor. Demo creation, which is the public write boundary, is proven live as above.

## Next operator action

- Register a recurring $79/month `clinic-reminder-proof` product in the Sociobot production and pilot catalogs, then run a real pilot checkout/return/verify cycle.
- Provide a non-root-writable durable data service (or a compatible managed database) and configure snapshot/restore automation. See `.factory/operations.md`.
- Verify the general rate limiter through the ingress’s trusted client-IP header, then enforce it at ingress or a shared limiter before increasing replicas.

---

# Previous independent verification 3 handoff

Status: **FAIL — do not release**

Date: 2026-08-28 UTC

Verifier work order: `clinic-reminder-proof-verify-3`

Candidate: `bc5592916143ff182424878a6cf60ef057d7007e`

Live URL: `https://clinic-reminder-proof.sociobot.in`

Fresh production evidence confirms that the candidate is deployed: `/health` reports the exact SHA and the live JS/CSS hashes match the clean local build. The mandatory cold first-read gate passes, all 17 installed clean-checkout claim commands pass, and the full local test/check/build gates pass.

Release is blocked by two live defects:

1. **Critical — checkout is unavailable.** The advertised Sociobot Clinic-plan checkout returns HTTP 404 with `{"error":"enabled factory product","status":404}`. A clinic cannot purchase the advertised $79/month plan.
2. **High — rate limits are replica-local.** A single client received 429 with `Retry-After`, then immediate 200 responses, then 429 again. A 120-request `/metrics` burst accepted 90 requests in 593 ms despite the configured burst of 40. The 5/hour creation and general governor state are held in each process, not across the live service.

Additional acceptance gaps: the broad managed-workflow claim checks route/config presence rather than completing provider and billing outcomes; live durable `/data` and backups remain unconfirmed; locally generated clinic key/database files are mode 0644.

Complete evidence, reproduction, passing gates, accessibility/mobile/performance results, and required fixes are in [verification-3.md](verification-3.md). No product code was changed by the verifier.

---

# Previous repair 2 handoff

Status: **implemented, deployed, and verified**

Date: 2026-08-28 UTC

Work order: `clinic-reminder-proof-repair-2`
Base verifier report: `4ee194d6ec349322b565900d36acc27855b40c40`
Failed candidate: `b7b2b9d615836b3aa9d708f058b24f9f30f390a2`

## Release-blocking repair

The verifier’s exact failure was reproduced first: `/start` was a browser-local CSV audit, while the public API had no authenticated clinic, connector, provider, receipt, exception, or billing workflow. The regression is now `@claim:managed-clinic-workflow` plus Rust `managed_claim_*` tests. It asserts that the CSV substitute is absent, public clinic routes reject missing bearer tokens with `401` and `WWW-Authenticate: Bearer`, and the only public identity configuration is the specified Sociobot Entra tenant/client.

The repair replaces that local substitute with a managed workflow:

- Microsoft Entra External ID PKCE sign-in using the required Sociobot tenant, session-storage cache, server-side discovery/JWKS validation, RS256/audience/tenant/issuer/time validation, and stable `oid` ownership.
- Encrypted service-side clinic store keyed by `oid`; patient destinations, provider credentials, connector secret, and billing entitlement are never returned in API responses or export. Data keys are CSPRNG-generated at first boot and persist beneath `DATA_DIR` (`/data` in the image).
- Signed calendar/EMR intake with a five-minute HMAC window, idempotent source-ID upsert, consent evidence, validated destinations, and ordered SMS/email/WhatsApp fallback policy.
- Real Twilio SMS/approved-WhatsApp and Resend email request adapters. Twilio callbacks use `X-Twilio-Signature`; Resend uses verified Svix (`whsec_`, `svix-id`, `svix-timestamp`, `svix-signature`) webhooks. Receipt event IDs are idempotent; terminal failure attempts the next consented configured channel, then opens a staff exception.
- Shared exception assignment/resolution, authenticated export/delete, and an authenticated same-origin checkout handoff that returns only the Sociobot billing URL. Billing entitlement is encrypted and revalidated before dispatch when older than one day.
- The demo remains isolated, simulated, and does not contact a provider. It never reads managed clinic data.

## Important safety decisions

- The demo is the only anonymous path. No real message is created from sample data.
- Dispatch requires a current Sociobot Clinic subscription, recorded channel consent, a configured approved provider/template, and an idempotency key. A consent block still opens a visible exception without a provider call.
- No clinical notes, diagnosis, treatment information, payment data, provider secret, or raw billing token is retained in a public browser store or export.
- Provider integrations were not sent live from this worker because no clinic-approved provider credentials or patient-consent records were supplied. The adapters and signed-receipt contracts are integration-tested with fixtures; activate them only with an approved clinic account.

## Exact regression coverage

| Finding / risk | Regression evidence |
| --- | --- |
| Local CSV substitute instead of product workflow | `@claim:managed-clinic-workflow` starts at `/start`, proves no file input or `real:` local-store key, validates exact CIAM metadata, public auth failures, workspace route, and billing boundary. |
| Missing durable, tenant-scoped clinic state | `managed_store_is_durable_and_tenant_scoped` reopens the encrypted store and proves `oid-a` cannot read `oid-b`. |
| Unsigned intake or provider proof | `managed_claim_clinic_flow_is_authenticated_signed_durable_and_consent_aware`, `managed_claim_twilio_signatures_are_checked_without_string_comparison`, and `managed_claim_resend_receipts_require_a_valid_svix_signature`. |
| Secret leakage | `connector_secret_is_not_an_audit_event_or_exported_workspace_field` and `managed_claim_provider_secrets_are_encrypted_at_rest`. |
| Unsafe fallback | `consent_guard_and_fallback_order_are_deterministic` and `managed_claim_terminal_failure_selects_the_next_untried_consented_channel`. |
| Existing M1 QA | All existing demo, privacy, limiter, 404, cache/security-header, offline, keyboard, mobile, and accessibility claim coverage remains in `tests/e2e/m1-claims.spec.ts`. |

## Verification evidence

Environment: Node 22.23.2, npm 10.9.8, rustc/cargo 1.98.0, Playwright 1.58.2.

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages, 0 vulnerabilities |
| `npm test` | PASS — 6 Vitest, 18 Rust, 22 Chromium tests (`test-results/.last-run.json` is `passed`) |
| `npm run test:managed-claim` | PASS — 5 managed Rust tests and the exact browser claim |
| `npm run check` | PASS — Svelte 0 diagnostics, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` |
| Public first-load assets | 80.08 KB JS raw / 27.90 KB gzip; 24.50 KB CSS raw / 5.31 KB gzip. The 271.99 KB MSAL chunk is lazy-loaded only when opening sign-in. |
| Browser / responsive / keyboard | PASS in the full Playwright suite: desktop, 390 px, 200% text, reduced motion, Tab/Enter skip link, back/deep-link, offline demo state, and visible 44 px targets. |
| Accessibility | PASS — Playwright axe checks both color schemes on `/`, `/demo`, `/start`, `/app`, `/privacy`, `/terms`, and unknown route; zero serious/critical findings. The requested standalone `@axe-core/cli` could not start Chrome in this container (`SessionNotCreatedError: cannot find Chrome binary`); Playwright used the supplied Chromium successfully. |
| Local release server | `verify-url.sh http://127.0.0.1:8081` PASS — HTTP 200, 655 ms, title/lang/one h1/main, zero missing alt and zero unlabelled buttons. Evidence: `.factory/qa-artifacts/repair-2/final/verify/verify.json`. |

## Run locally

```sh
npm ci
npm test
npm run test:managed-claim
npm run check
npm run build
PORT=8080 DIST_DIR=dist target/release/reminder-proof-api
```

Use `/?demo=1` for the isolated sample. Use `/start` for Entra sign-in, then `/app` for a managed workspace.

## Deployment and operator action

Deploy class remains **container** using `Dockerfile`, port 8080, to `https://clinic-reminder-proof.sociobot.in`. The deployed product revision is `ee9bf6366d68f852c3ee65dfde3d52b1f594975a`; Azure ingress and the public hostname both returned that exact `build_sha`. The final public `verify-url.sh` check passed in 644 ms with no console errors and the expected title/lang/h1/main/alt/button evidence.

Before a production clinic can finish Entra login, the factory must confirm that `https://clinic-reminder-proof.sociobot.in/auth/callback` is registered on the shared SPA application. The factory must also register the monthly `clinic-reminder-proof` Sociobot subscription product and ensure the container has persistent `/data` storage before accepting real clinic records. These are deployment/operator configuration, not values stored in the repository. Provider credentials, webhook secrets, BAA/DPA review, approved templates, jurisdiction messaging rules, and patient consent remain each clinic’s responsibility.
