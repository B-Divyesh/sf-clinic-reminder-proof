# Independent product verification 12 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-12`
Candidate: `a95a64b6f1ccba175adc34dec75a23ad38bb8974`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The candidate source builds and its local product tests pass, but the deployed candidate revision cannot start safely. Azure reports candidate revision `sf-clinic-reminder-proof--0000039` as `ActivationFailed`, with 100% traffic configured, `minReplicas: 1`, `maxReplicas: 3`, and no durable/backup volume mounts. Its container log gives the direct cause: the application's production mount guard exits because `/durable` and `/backups` are absent.

The public URL still answers from the earlier healthy build `aae2ef63533d5daf18c175feabb03aa482152d9f`, not candidate `a95a64b6f1ccba175adc34dec75a23ad38bb8974`. This is a deployment-only failure reproduced from fresh, read-only Azure, HTTP, and container-log evidence.

## Required gates

| Gate | Result | Evidence |
| --- | --- | --- |
| Candidate checkout | PASS | Clean `HEAD` was `a95a64b6f1ccba175adc34dec75a23ad38bb8974`. |
| Clean install | PASS | `npm ci`: 87 locked packages installed; audit reported 0 vulnerabilities. |
| Claims manifest | PASS locally | `.factory/claims.json` exists with 31 unique claims. Every listed command was run separately in manifest order against the local demo/API entry point; the complete independent claim suite passed. |
| Unit/integration/browser suite | PASS | `npm test`: 9 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type/lint/format | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt and Clippy (`-D warnings`) passed. |
| Production build | PASS | `npm run build` emitted `dist/` and the optimized API binary. Initial entry JS is 28.63 KB gzip; CSS is 5.54 KB gzip. |
| Deployment verification | **FAIL** | `EXPECTED_BUILD_SHA=a95... npm run verify:deployment` failed: `maximum replica count; expected 1, got 3`. |
| Candidate is live | **FAIL** | Public `/health` returned `build_sha: aae2ef63533d5daf18c175feabb03aa482152d9f`, not `a95...`. |

## First read and demo

**PASS.** A cold 1440×900 public visit answered all required first-screen questions in plain words:

- What it does: “See every reminder outcome.”
- For whom: independent clinics needing delivery proof and a clear next step when reminders fail.
- What to do first: visible “Try it with sample data”.
- What follows: “Opens a sample clinic. Nothing touches real clinic data.”

The one-click action opened `/demo`, showed the persistent sample-data banner, and exposed Reset demo and Start for real. Fresh desktop and 390 px mobile sessions exercised advancing due reminders, assignment, resolution, undo, and reset. The mobile page had no horizontal overflow.

## Release-blocking finding

### QA12-01 — Critical — candidate deployment is unhealthy, lacks required durable mounts, and the public site is not the candidate

Fresh Azure inspection found:

```json
{
  "applicationTemplate": {
    "minReplicas": 1,
    "maxReplicas": 3,
    "volumes": null,
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:a95a64b6f1cc"
  },
  "candidateRevision": {
    "name": "sf-clinic-reminder-proof--0000039",
    "trafficWeight": 100,
    "healthState": "Unhealthy",
    "runningState": "ActivationFailed",
    "replicas": 1,
    "maxReplicas": 3,
    "volumes": null
  },
  "previousRevision": {
    "name": "sf-clinic-reminder-proof--0000038",
    "trafficWeight": 0,
    "healthState": "Healthy",
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:aae2ef63533d",
    "replicas": 1,
    "maxReplicas": 1,
    "volumes": ["clinic-data", "clinic-backups"]
  }
}
```

The candidate container starts, identifies itself as `a95...`, then panics safely:

```text
initialize durable clinic store: "required durable storage mounts are missing: /durable, /backups; refusing unsafe production storage"
```

The checked-in `deployment/containerapp.json` requires exactly one replica and Azure Files mounts at `/durable` and `/backups`. A backend that stores managed clinic workspaces, encryption keys, and recovery pairs cannot safely run the candidate deployment without those mounts; three permitted replicas would also invalidate its process-local rate-limit and single-owner topology contract.

The live site consequently serves the previous revision: `GET /health` returned `{"status":"ok","build_sha":"aae2ef63533d5daf18c175feabb03aa482152d9f"}`. Although `a95...` changes only `.factory/handoff.md` relative to `aae...`, the requested live-candidate identity check is explicit and fails.

Required repair: deploy a healthy revision built from `a95...` (or its intended successor) with `minReplicas=maxReplicas=1`, the two checked-in Azure Files mounts, and 100% traffic only after readiness. Then rerun `EXPECTED_BUILD_SHA=<deployed full SHA> npm run verify:deployment` and confirm `/health` and the footer identify that exact build.

## Live behavior, privacy, auth, accessibility, and headers

These checks describe the older healthy public revision only; they do not cure QA12-01.

- A fresh landing-to-demo request log contained only `https://clinic-reminder-proof.sociobot.in` requests. No analytics, provider, billing, or other third-party runtime request loaded. No console or page errors occurred.
- Desktop and 390 px live Axe scans had **zero serious or critical findings**. Keyboard Tab exposed a 3 px `#005fcc` focus outline, including the skip link; the reduced-motion rule reduced animation/transition durations to `0.01ms`.
- Root and health responses send CSP including response-header `frame-ancestors 'none'`, HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, permissions policy, and COOP. HTML is `no-cache`; hashed live JS is `public, max-age=31536000, immutable`.
- The public sign-in route uses only the required Sociobot Microsoft Entra External ID authority: `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/oauth2/v2.0/authorize`, client `25c704f4-465a-47af-80ab-2c489466b697`, PKCE, and `/auth/callback`.
- Invalid API recovery paths worked on the old live revision: malformed JSON returned 400, wrong content type returned 415, and authenticated checkout without a bearer token returned 401 with `WWW-Authenticate: Bearer`; each was structured JSON with matching UUID request IDs.
- The documented demo-creation allowance was enforced on the old healthy revision: six POSTs using one fixed first `X-Forwarded-For` hop returned `200, 200, 200, 200, 200, 429`; request six carried `Retry-After: 3599` and `code: rate_limited`.

## Scope notes

This is a web product with a backend, not a library, CLI, or PWA; package-consumer, CLI, and service-worker update checks do not apply. No real clinic data, patient data, credentials, payment, or authenticated clinic account was used. No product code was modified.

## Defects by severity

| Severity | Finding |
| --- | --- |
| Critical | QA12-01: candidate revision `0000039` fails activation because required durable mounts are absent; it allows three replicas; the public URL remains the older `aae...` build. |
| High | None found beyond the release-blocking deployment failure. |
| Medium | None found in the tested older live revision. |
| Low | None. |
