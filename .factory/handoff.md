# M2 handoff — Reminder Proof

Date: 2026-08-30 UTC

Work order: `venture-clinic-reminder-proof-m2`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status

M2 application work is deployed and verified. Independent review/polish and external Sociobot pilot-catalog enablement remain before the milestone can be marked complete.

## Delivered

- Real shared-tenant Entra CIAM PKCE sign-in and strict backend JWT/JWKS validation.
- Reversible normalized SQLite account migrations, tenant/role boundaries, durable backup pairs, and one-replica deployment enforcement.
- Clinic, location, jurisdiction, retention, and staff onboarding.
- Three monthly plan choices wired through a rate-limited same-origin API to the Sociobot pilot gateway, with encrypted entitlements and complete subscription-state handling.
- Owner export and audited, cancellable seven-day account deletion.
- Six M2 claim tests, route-specific titles, mobile/keyboard/axe coverage, and an unchanged isolated demo.

The planned PostgreSQL database was corrected to single-writer SQLite because the factory runtime supplies only `PORT`. `.factory/plan.md` records the reason and requires PostgreSQL before scaling beyond one replica.

## Verification

```sh
npm ci
npm test
npm run check
npm run build
```

Fresh-clone results: 21 Vitest, 41 Rust, and 47 Chromium tests passed. All 37 claim commands passed separately. The live deployment verifier confirmed one replica, durable mounts, exact build identity, `429` plus `Retry-After`, and 100% traffic to the healthy revision.

Cold live checks passed for the landing page, demo, and all M2 routes with no console errors or serious/critical axe findings. Lighthouse mobile scored 99 Performance and 100 Accessibility, Best Practices, and SEO; LCP was 1.5 s and CLS 0.001.

## Needs operator action

Enable `clinic-reminder-proof` and its Clinic, Practice, and Network recurring tiers in the Sociobot pilot billing catalog. The gateway currently returns 404 `enabled factory product`, so a real hosted test checkout cannot complete. After enablement, run one pilot subscription, return, cancellation, and revocation flow.

No customer credentials were available for an interactive Entra login. CIAM discovery, the registered production callback, PKCE wiring, and backend validation were verified without bypassing authentication.

Detailed evidence and M3 requirements are in `.factory/handoff-m2.md`.
