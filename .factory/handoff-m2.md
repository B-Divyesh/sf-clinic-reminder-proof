# M2 builder handoff — accounts, persistence, and billing

Work order: `venture-clinic-reminder-proof-m2`

Date: 2026-08-30

Status: implementation deployed with immutable image verification. Pilot checkout remains blocked by external product-catalog enablement.

## What shipped

- Shared Sociobot Entra CIAM sign-in through MSAL authorization code + PKCE. Session tokens stay in `sessionStorage`. The API loads OIDC discovery and JWKS, validates RS256, issuer, audience, tenant, expiry, and `nbf`, and keys users by `oid`.
- Reversible SQLite migrations for users, organizations, locations, memberships, subscriptions, audit events, notification preferences, and export jobs. Foreign keys, organization-scoped reads, server-side roles, encrypted sensitive fields, durable snapshots, and daily backup pairs are covered by API tests.
- Three-step onboarding for clinic details, jurisdiction, retention, first location, timezone, owner, and staff roles. Clinical-note guidance is explicit.
- Clinic, Practice, and Network plan selection through the same-origin billing API. The server defaults to the Sociobot pilot gateway, validates tiers, keeps entitlement tokens encrypted, caches verification for one day, and handles active, grace, past-due, cancelled, revoked, and unavailable states.
- Owner-only JSON export, audited deletion scheduling, cancellation during a seven-day recovery window, and automatic purge after the window. The older clinic deletion endpoint now schedules the same recovery window and cannot bypass it.
- M2 routes, exact titles, responsive layouts, keyboard/focus behavior, loading/error states, and the existing translucent pulse-ledger design system. The public one-click demo remains isolated and resettable.
- Six M2 entries in `.factory/claims.json`: `ciam-sign-in`, `tenant-isolation`, `durable-onboarding`, `subscription-price`, `data-export`, and `account-deletion`.

## Plan correction

The original architecture required PostgreSQL, but the deployment contract supplies only `PORT` and no database service or connection URL. M2 therefore uses normalized SQLite tables on the existing durable Azure Files topology with exactly one writer replica. The migration is reversible and the plan now requires PostgreSQL before horizontal API or worker scaling. This is recorded in `.factory/plan.md`; it was not a silent deviation.

## Verification evidence

All source checks below passed from a fresh shallow clone of pushed `main`:

| Check | Result |
| --- | --- |
| `npm test` | Pass — 21 Vitest, 41 Rust, 47 Chromium |
| `npm run check` | Pass — zero Svelte diagnostics; rustfmt and clippy clean |
| `npm run build` | Pass — `dist/` and release API binary produced |
| Every `.factory/claims.json` command | Pass — 37 of 37 run separately from the clean clone |
| M2 route axe checks | Pass — no serious or critical findings at 390 px |
| Initial web assets | App JS 31.82 KB gzip; CSS 5.79 KB gzip; lazy MSAL chunk 68.23 KB gzip |

The application release was built in ACR as a full-SHA image and deployed through the repository's topology-preserving Container Apps command. The verifier confirmed one healthy replica, both durable mounts, 100% traffic, exact image/build identity, and a sixth same-client request returning `429` with `Retry-After`. A separate live billing-route smoke returned five `401` responses followed by `429` with `Retry-After: 119`.

Cold production evidence at <https://clinic-reminder-proof.sociobot.in>:

- Landing and demo return 200 with no browser console errors, one h1, `lang=en`, a main landmark, and no missing alt text or unlabeled button.
- The demo title is `Demo — Reminder Proof`; all M2 route title, mobile reflow, and axe checks pass against production.
- Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100, SEO 100, LCP 1.5 s, CLS 0.001, TBT 90 ms.
- CIAM discovery returned the tenant-guid issuer and shared JWKS. The production authorization request returned 200 with no callback mismatch for `/auth/callback`.

## Needs operator action

The live pilot gateway returns HTTP 404 with `{"error":"enabled factory product","status":404}` for `/api/v1/products/clinic-reminder-proof/checkout`. This work order has no billing-catalog credential or registration command. The factory must enable the slug and its three recurring tiers, then complete one hosted test subscription and cancellation using the pilot card. Until then, the app shows the gateway error and preserves read, export, and safety actions; it does not pretend checkout worked.

No customer credentials were available for a full interactive Entra sign-in. Discovery, callback registration, frontend PKCE wiring, and backend token validation were verified without weakening authentication.

## M2 build 1 deployment repair

On 2026-09-05, implementation `1e543b6d6d25997267a1af48586f17609d3b0eeb`
was built through the corrected fleet wrapper and deployed as
`sociobotregistry.azurecr.io/sf-clinic-reminder-proof@sha256:cb3684c090e566fc46864c3617b66e203043f5a00f5f029f174580fc682a5e3b`.
The active revision is `sf-clinic-reminder-proof--0000069`. It is healthy,
solely traffic-bearing, and keeps the existing `clinic-data` `/data` and
`clinic-backups` `/backups` mounts with exactly one replica. The local
deployment verifier now checks that immutable digest form and passed against
the public health/footer build identity and five-then-429 allowance.

The pilot checkout dependency was intentionally retained. The public Clinic
offer is $79 per location each month; the pilot catalog still needs all three
recurring tiers registered before a clinic can pay and activate live dispatch.

## M2 build 1 reset repair

Implementation `ffebafb4a2d6c243424f0750f6c18c1b840df079` fixed a live demo
recovery defect found during the repair verification. Resetting sample data had
incorrectly counted as a new-demo allocation, which could make Reset demo
return `429` after repeated use. Reset now reseeds a new fictional session
without consuming the five-per-hour new-demo allowance. The outcome regression
creates five allowed demos around ten resets and confirms only a sixth new demo
is limited.

It is deployed in revision `sf-clinic-reminder-proof--0000070` as
`sociobotregistry.azurecr.io/sf-clinic-reminder-proof@sha256:b93cad45e9dcde5e2e2ac9316e066d73db6b5de0480a9c5b19a7ae1f7d50c50f`.
Fresh desktop and 390 px phone browsers each completed the one-click sample,
advance, assign, resolve, reload, and reset journey with no console errors or
third-party requests. The pilot billing dependency is unchanged.

## What M3 needs

1. Run the independent M2 review and polish loop before marking M2 complete.
2. Enable the pilot billing product and verify the paid return, duplicate return, cancellation, and revocation against the live gateway.
3. Use a consented pilot clinic to validate the already-present signed intake and provider workflows with test credentials; never use the public demo for a provider call.
4. Move persistence and leases to PostgreSQL before adding another API or worker replica.
