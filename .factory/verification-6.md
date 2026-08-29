# Independent product verification 6 — FAIL

Date: 2026-08-29 UTC  
Work order: `clinic-reminder-proof-verify-6`  
Candidate: `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d`  
Live URL: https://clinic-reminder-proof.sociobot.in

## Verdict

**FAIL — do not release or accept real clinic data.** The former deployment-identity concern is resolved: live `/health` reports the exact candidate SHA and Azure reports image `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:a3ec1d2b5a24`.

However, fresh Azure control-plane evidence shows the active revision is not using the checked-in single-replica, durable-storage topology. Its active configuration permits three replicas and has neither durable nor backup volume mounts. That makes the claimed managed workspace persistence and service-wide rate limit unsafe when the platform scales it.

## First-read and demo gates — PASS

A cold browser visit, before interaction, said:

- **What:** “See every reminder outcome.”
- **For whom:** “For independent clinics that need delivery proof and a clear next step when reminders fail.”
- **What to click first:** visible **“Try it with sample data”**, with “Opens a sample clinic. Nothing touches real clinic data.”

The one-click action opened `/demo` with five fictional appointments, delivery evidence, simulated fallback, and an assignable exception. This meets the plain-words and isolated-demo entry requirements.

## Release-blocking finding

### Critical — active production topology has no persistent storage and can scale to three state owners

Read-only Azure inspection of active revision `sf-clinic-reminder-proof--0000024` returned:

```json
{
  "activeRevisionsMode": "Single",
  "scale": { "minReplicas": 1, "maxReplicas": 3 },
  "volumes": null,
  "mounts": null,
  "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:a3ec1d2b5a24"
}
```

One replica was running at the instant checked, but the app can scale to three. The current active revision has no Azure Files mount at `/durable` or `/backups`. Its local SQLite state and recovery files are therefore container-local and can be lost or split across replicas. This contradicts the live promises in the README and operations guide, plus the `managed-auth-storage`, `managed-storage-recovery`, and `single-replica-durable-topology` claims.

It also invalidates process-local rate-limit guarantees at scale. A fresh live probe with a stable first `X-Forwarded-For` hop currently admitted exactly five concurrent demo creations and rejected 13 of 18 with `429` and `Retry-After: 3599`; that is correct only while one process serves the app. If three replicas are used, the same client can receive up to three separate allowances.

**Required repair:** deploy revision 24 (or a replacement) with `minReplicas: 1`, `maxReplicas: 1`, both Azure Files volumes, and `/durable` / `/backups` mounts exactly as in `deployment/containerapp.json`. Then prove a signed-in workspace survives a replica replacement and backup restore. Do not increase replica count until storage and rate-limit state move to shared services.

## Mandatory claims gate — PASS locally

From a clean worktree at the candidate SHA, after `npm ci` (87 packages, zero reported vulnerabilities), every exact command in `.factory/claims.json` was run individually before the broader suite. All passed, including the cold first command, which compiled the Rust server in 2m16s and finished in 2m24s.

| Claims | Result |
| --- | --- |
| `demo-isolation` through `build-identity` (16 demo/public claims) | PASS |
| `managed-auth-storage` | PASS (Rust fixture + browser contract) |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS against checked-in JSON, **not live** |

The last result is not sufficient to accept the candidate because Azure shows the checked-in topology was not applied.

## Local quality gates — PASS

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages; 0 vulnerabilities |
| `npm test` | PASS — 6 Vitest, 24 Rust, 28 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/warnings; rustfmt; clippy `-D warnings` |
| `npm run build` | PASS — Vite `dist/` and release API binary |
| Runtime with only `PORT=18086` | PASS — generated default local store/key; `/health` `200` and Prometheus `/metrics` |

The public entry JS is 80,137 bytes raw / 27,503 gzip and CSS is 24,799 / 5,396 gzip. The 271,994-byte MSAL chunk is lazy, not requested on landing or demo. This is within the initial-load budget.

## Live functional, privacy, security, and accessibility evidence

- Landing → Demo → advance due reminders → assign Sofia R. to Sam Rivera → resolve → reload (owner persisted) → undo → reset was completed on the live site. The entire flow made same-origin requests only; no console or page errors occurred.
- Fresh production rate probe observed **5 demo-workspace creates per client per hour**, then `429` JSON plus `Retry-After: 3599`. A separate protected billing burst returned 44 `401` and 16 `429`, each rejection carrying `Retry-After: 1`.
- `rp_demo` is HttpOnly, Secure, SameSite=Lax, scoped to the demo API, and has `Max-Age=86400` (covered by the passing claim test).
- Browser responses send CSP with response-header `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP. The live hashed JS asset sends `public, max-age=31536000, immutable`.
- `/api/v1/auth/config` names only the required `sociobotcustomers.ciamlogin.com` Entra authority; unauthenticated clinic and export routes return `401` with `WWW-Authenticate: Bearer`.
- The production Sociobot product checkout now works: its public checkout boundary returned `303` to `checkout.dodopayments.com` for the advertised $79/month Clinic plan.
- Desktop and 390px demo flows worked. At 390px with 200% root text, `/`, `/demo`, `/privacy`, `/terms`, `/start`, `/app`, and `/404` each measured `scrollWidth == clientWidth == 390`. After keyboard-equivalent resolution, focus moved to “Undo resolution.”
- Playwright axe found zero serious or critical findings on live desktop and 390px reduced-motion demo; reduced-motion reported no active animations.

## Next steps

1. Apply the checked-in Container App volume and scale configuration to the active revision, then independently recheck it with Azure and live bursts.
2. Prove real authenticated workspace persistence through a replacement and recovery from the mounted backup pair.
3. Re-run this verification; the product code and local test suite need no change for the topology defect.

No product code was modified during verification.
