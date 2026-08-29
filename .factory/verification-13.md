# Independent product verification 13 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-13`
Candidate: `4ee36ffaa1496e94ebdb0f0b0fc17b907f824372`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The source candidate is locally healthy and its public runtime reports the requested build SHA. The traffic-bearing Azure revision, however, is not deployed with the candidate's mandatory one-replica durable-storage topology. This is a production-only, release-blocking defect.

## Required first checks

### First-read and one-click demo: PASS

A cold production page answered all required questions in plain words:

- **What:** “See every reminder outcome.”
- **For whom:** independent clinics needing delivery proof and a clear next step when a reminder fails.
- **First action:** visible **Try it with sample data**, immediately explained as opening a sample clinic without touching real clinic data.

The action opened `/demo` in one click. It showed the persistent **“Demo — sample data, nothing is saved to your clinic”** banner with **Reset demo** and **Start for real**.

### Claims gate: FAIL (30/31 exact manifest commands passed)

`.factory/claims.json` exists. From the clean detached candidate checkout, after `npm ci`, I ran every literal `test` command in manifest order, separately, against the shipped demo entry point. The retained output is `/tmp/clinic-claims.wRYaJc/claims.log` and the per-claim status file is `/tmp/clinic-claims.wRYaJc/results.tsv`.

- The first 30 IDs passed: `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline`, `exception-ownership`, `sample-exception-visibility`, `demo-reset`, `minimal-reminder-content`, `public-price`, `demo-cookie-lifetime`, `demo-replica-continuity`, `no-tracking`, `explicit-theme-choice`, `request-protection`, `rate-limit-policy`, `security-headers`, `build-identity`, `managed-auth-storage`, `signed-calendar-intake`, `approved-whatsapp-dispatch`, `twilio-receipt-verification`, `resend-receipt-verification`, `managed-secret-encryption`, `managed-data-minimisation`, `no-marketing-campaigns`, `signed-in-export-delete`, `managed-provider-fallback-receipt`, `managed-billing-return`, and `managed-storage-recovery`.
- `single-replica-durable-topology`: its browser portion passed, but the required trailing `npm run verify:deployment` failed: `maximum replica count; expected 1, got 3`.

Because a declared claim command failed, this gate is release-blocking.

## Local candidate checks

| Check | Result | Evidence |
| --- | --- | --- |
| Clean install | PASS | `npm ci`: 87 locked packages, 0 vulnerabilities reported. |
| Full tests | PASS | `npm test`: 11 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type, format, lint | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt; Clippy with `-D warnings`. |
| Production build | PASS | `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. Initial JS was 28.63 KB gzip and CSS 5.54 KB gzip. |
| Runtime defaults | PASS | Release binary started with only `PORT=18081`, served `/health`, and logged generated durable-key configuration without revealing a secret. |

## Live functional, privacy, and accessibility evidence

- Live `/health` returned `200` and `{"status":"ok","build_sha":"4ee36ffaa1496e94ebdb0f0b0fc17b907f824372"}`; the footer displayed `Build 4ee36ff`.
- In a fresh live demo, advancing due reminders produced evidence, the staff exception was assigned to Sam Rivera and resolved, then Reset demo restored the four-due seed. No console or page errors occurred.
- Cold landing and demo request logs contained only `https://clinic-reminder-proof.sociobot.in` resources/API calls. Fonts, CSS, and JavaScript were self-hosted; no tracking or messaging-provider request loaded.
- Desktop and 390 px mobile layouts worked. Keyboard traversal reached the skip link, navigation, theme selector, demo action, and footer; desktop focus used a visible 3 px `rgb(0,95,204)` outline. Reduced-motion emulation completed without page errors. Axe (desktop and mobile) reported zero serious or critical violations.
- Headers included CSP with response-header `frame-ancestors 'none'`, HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, permissions policy, and COOP. The hashed live JS asset used `Cache-Control: public, max-age=31536000, immutable`.
- `/api/v1/auth/config` exposes only the required Sociobot Entra External ID authority, and an anonymous clinic request returns `401` with `WWW-Authenticate: Bearer`.
- Live demo-create allowance: five `POST /api/v1/demo/workspaces` requests for one stable first `X-Forwarded-For` hop returned 200; request six returned `429` JSON with `Retry-After: 3599`. A 60-request concurrent protected-endpoint probe returned 43×401 and 17×429; the 429 responses had `Retry-After: 1` (the documented generic governor burst is 40 and refill can occur during the probe).

## Release-blocking finding

### QA13-01 — Critical — traffic is on a three-replica, non-durable revision; the correct candidate revision has 0% traffic

`npm run verify:deployment` failed before acceptance. Independent read-only Azure inspection showed:

```json
{
  "candidateRevision": {
    "name": "sf-clinic-reminder-proof--0000040",
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:4ee36ffaa1496e94ebdb0f0b0fc17b907f824372",
    "trafficWeight": 0,
    "scale": {"minReplicas": 1, "maxReplicas": 1},
    "mounts": ["/durable", "/backups"],
    "volumes": ["clinic-data", "clinic-backups"]
  },
  "trafficBearingRevision": {
    "name": "sf-clinic-reminder-proof--0000041",
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:85fbd01045e2",
    "trafficWeight": 100,
    "scale": {"minReplicas": 1, "maxReplicas": 3},
    "mounts": [],
    "volumes": []
  }
}
```

The runtime build identity at the public URL says `4ee36…`, but Azure's serving revision is tagged differently and, critically, lacks both required Azure Files mounts and permits three replicas. The repository's checked-in `deployment/containerapp.json` requires exactly one replica and the `clinic-data`/`clinic-backups` mounts. Without that topology, managed clinic workspaces, encryption key material, recovery pairs, and process-local rate-limit state are not safely single-owner or durable.

**Repair:** deploy the candidate image and `deployment/containerapp.json` as one revision, ensure that revision alone carries 100% traffic, verify its image ends in `:4ee36ffaa1496e94ebdb0f0b0fc17b907f824372`, and rerun `EXPECTED_BUILD_SHA=4ee36ffaa1496e94ebdb0f0b0fc17b907f824372 npm run verify:deployment` until it passes.

## Defects by severity

| Severity | ID | Summary |
| --- | --- | --- |
| Critical | QA13-01 | Serving Azure revision allows three replicas and has no durable/backup mounts; candidate topology is idle at 0% traffic. |
| High / Medium / Low | — | No additional independently reproducible defects found in this run. |

## Reproduce local gates

```sh
npm ci
npm test
npm run check
npm run build
EXPECTED_BUILD_SHA=4ee36ffaa1496e94ebdb0f0b0fc17b907f824372 npm run verify:deployment
```
