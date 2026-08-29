# Independent verification 7 handoff — FAIL

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-7`

Candidate and live build: `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d`

Live URL: https://clinic-reminder-proof.sociobot.in

**Status: FAIL — do not release until the claims contract is repaired.** Fresh
evidence confirms the earlier deployment-only failure is resolved: production
runs the exact candidate as one healthy replica with both Azure Files mounts,
and concurrent probes enforce five demo creations per client-hour plus the
40-request shared API burst, all with `Retry-After` on 429 responses.

All 21 declared claim commands pass after `npm ci`; `npm test` passes 6 Vitest,
24 Rust, and 28 Chromium tests; `npm run check` and `npm run build` pass. The
cold first-read, one-click demo, normal and invalid recovery flows, live
same-origin request log, Entra redirect, hosted $79/month checkout, desktop and
390 px keyboard/reduced-motion behavior, live axe scan, headers, asset hashes,
and mobile Lighthouse (100/100/100/100; LCP 1.2 s, CLS 0) were verified.

Two claims-contract defects still block acceptance:

1. The `request-protection` claim promises JSON and 16 KB limits with
   structured errors for “API writes,” but wrong content type returns plain
   text 415 and protected clinic/billing writes use a 5 MB limit. The listed
   test only covers demo malformed JSON and an oversized demo request.
2. The landing/privacy/README promises that the product sends no marketing
   campaigns and that signed-in clinics can export/delete workspaces, but
   neither promise has a corresponding `.factory/claims.json` entry and
   observable claim test.

Also fix misleading API request IDs (missing on 401s, constant on normalized
errors), the missing explicit theme choice promised by `design.md`, and the
duplicate description meta element. Full commands, evidence, severities, and
limitations are in [verification-7.md](verification-7.md). No product source
code was changed.

---

# Verification handoff — FAIL

Candidate `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d` at https://clinic-reminder-proof.sociobot.in **FAILS release verification**.

The live `/health` SHA and Azure image match the candidate. All 21 declared claim commands, `npm test`, `npm run check`, and `npm run build` pass locally; the live demo, privacy/network checks, rate-limit probe, accessibility, mobile reflow, and hosted checkout also pass.

Release remains blocked by the deployed Container App configuration: active revision `sf-clinic-reminder-proof--0000024` has `maxReplicas: 3`, no volumes, and no `/durable` or `/backups` mounts. Managed clinic data and recovery files are therefore ephemeral and replica-local, and process-local client limits will multiply if scaled. Do not accept real clinic data.

Required operator action: deploy the checked-in single-replica configuration with the two Azure Files mounts, then prove authenticated persistence across a replacement and backup restoration. See `.factory/verification-6.md` for the exact commands, observed 5-per-hour demo allowance, and full evidence.
