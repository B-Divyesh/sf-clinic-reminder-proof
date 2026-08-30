# Verification handoff — Reminder Proof

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-verify-22`

Candidate: `d65c4001e076b07740b265c04165df2008084595`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status

**FAIL — do not release.** Public pages and the demo serve the candidate and pass functional, accessibility, privacy, and performance checks. Release is blocked by the failed production topology claim, unavailable paid checkout, and a spoofable per-client rate-limit key.

## Release blockers

1. Azure revision `sf-clinic-reminder-proof--0000063` is the active 100%-traffic target but is `Unhealthy` / `ActivationFailed`. It uses a short image tag, `maxReplicas: 3`, and no durable or backup mounts. Public traffic falls back to healthy candidate revision `0000062`. `npm run verify:deployment:current` fails.
2. The exact Sociobot checkout URLs used by the backend return HTTP 500 for Clinic, Practice, and Network on both pilot and production gateway hosts. Because live dispatch requires an active subscription, a new clinic cannot complete the real job.
3. The nominal allowance is five demo creations per client per hour, then 429 with `Retry-After`. The same network client bypassed it by varying caller-supplied first-hop `X-Forwarded-For`; seven of seven requests returned 200. The general and billing limiters share this trust defect.

## Verification summary

- Mandatory claims: 36/37 passed; `single-replica-durable-topology` failed on fresh live Azure state.
- First-read and one-click sample demo: pass at desktop and 390 px.
- `npm ci`: pass, 87 packages, 0 vulnerabilities.
- `npm test`: pass, 21 Vitest + 41 Rust + 47 Chromium.
- `npm run check`: pass, zero Svelte diagnostics, rustfmt and Clippy clean.
- Exact full-SHA `npm run build`: pass; `dist/` and release binary produced.
- Complete live browser suite: 47/47 pass.
- Public build identity: health and footer match the candidate; local/live entry JS SHA-256 matches.
- Live demo: 4 due → 3 delivered + 1 exception; fallback, assignment, resolution persistence, undo, and reset work.
- Privacy: 18/18 observed demo-flow requests were same-origin; no console/page/request errors.
- Accessibility: no serious/critical axe findings; 390 px reflow, 44 px targets, skip focus, visible 3 px focus ring, and reduced motion pass.
- Lighthouse mobile: 99 Performance, 100 Accessibility, 100 Best Practices, 100 SEO; LCP 1.4 s, TBT 140 ms, CLS 0.001.
- Default runtime: starts with only `PORT`; 100 concurrent health requests passed.
- Authentication: actual redirect uses only the shared Sociobot CIAM tenant, correct client/callback, code flow, and PKCE S256.
- Container build was not run because Docker and Podman are unavailable in the verifier image.

## Required next steps

1. Redeploy the full candidate SHA through the topology-preserving repository command. Require one healthy, full-traffic revision with `minReplicas=maxReplicas=1` and both Azure Files mounts.
2. Enable all offered recurring tiers in Sociobot billing and complete a real hosted test purchase, return, cancellation, and revocation.
3. Establish a trusted client-IP boundary at ingress and reject or overwrite caller-supplied forwarding chains; add a live bypass regression.
4. Rerun every `.factory/claims.json` command, `npm test`, `npm run check`, the exact full-SHA build, the live browser suite, and `npm run verify:deployment:current`.

Full evidence: [verification-22.md](verification-22.md).
