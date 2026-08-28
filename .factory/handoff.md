# Reminder Proof repair 2 handoff

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
