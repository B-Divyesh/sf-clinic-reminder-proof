# Verification 11 handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-11`
Candidate: `4b07dd38cae3bb33530eca8704aff3f9b243cbfb`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL

Do not release or accept real clinic data. The exact candidate is live and the application code passes its local gates, but production is running three replicas with no durable or backup volume mounts. This makes clinic state and generated keys container-local and multiplies the per-client rate allowance. Five mobile demo evidence links also miss the 44 px touch-target requirement.

Full evidence and reproduction details are in [verification-11.md](verification-11.md).

## Release blockers

1. **Critical — deployment topology:** active revision `sf-clinic-reminder-proof--0000037` reports image `4b07dd38cae3`, three replicas, `maxReplicas=3`, `volumes=null`, and `mounts=null`. The repository requires one replica plus Azure Files at `/durable` and `/backups`.
2. **Observed enforcement failure:** one client received 15 successful demo-workspace creations; only request 16 returned 429 with `Retry-After: 3599`. The documented allowance is five, so request six must return 429.
3. **Medium — mobile target size:** all five demo **View evidence** links measure 92×18 CSS px at 390 px, below the required 44×44 px.

## What passed

- Cold first-read and one-click sample demo gate.
- Clean install and all 31 declared claim commands locally.
- `npm test`: 9 Vitest, 33 Rust, 39 Playwright.
- `npm run check`: Svelte, rustfmt, and Clippy.
- `npm run build`: web `dist/` and release API.
- Live `/health` and local/live JS/CSS hashes match the candidate.
- Demo job flow, invalid-input recovery, sign-in redirect, privacy request log, headers, caching, responsive reflow, reduced motion, keyboard skip path, and zero serious/critical axe findings.
- Mobile Lighthouse: 99 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.50 s, TBT 115 ms, CLS 0.00074.

## Required next steps

1. Apply `deployment/containerapp.json` to a new production revision: exactly one replica and both Azure Files mounts.
2. Confirm the active revision itself—not only the checked-in file—shows `minReplicas=1`, `maxReplicas=1`, `/durable`, and `/backups` with 100% traffic.
3. Re-run the declared live rate claim and confirm requests 1–5 are accepted and request six is 429 with `Retry-After`.
4. Prove a signed managed fixture persists through replica replacement and restores from the mounted backup pair.
5. Increase the five ledger evidence-link hit areas to at least 44×44 px and add an all-controls mobile target-size test.
6. Repeat independent verification before onboarding a clinic.

## Reproduce local gates

```sh
npm ci
npm test
npm run check
npm run build
```

No product code was modified. Only this handoff and `.factory/verification-11.md` were added/updated by the verifier.
