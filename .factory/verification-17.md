# Independent product verification 17 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-17`
Candidate: `3a341b7d34f6e734791ec37596295cda193374ed`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The candidate is locally
healthy, but it is not serving the public URL. Its Azure revision is unhealthy
while declared at 100% traffic because the rollout discarded the mandatory
single-replica durable-storage topology. Public traffic falls back to the older
healthy `2ec97d29…` revision.

## Mandatory first gates

### Claims: FAIL as a release gate

`.factory/claims.json` is present with 31 entries. From this clean checkout I
ran the complete tagged browser claim set through the shipped demo entry point:

```text
CI=1 npm run test:e2e -- --grep @claim:
31 passed (47.3s)
```

The three additional Rust commands named by the manifest also passed:
`managed_claim_clinic_flow_is_authenticated_signed_durable_and_consent_aware`,
`managed_claim_no_marketing_campaigns`, and
`managed_claim_export_and_delete_are_owner_scoped`.

The final manifest entry is composite and requires
`npm run verify:deployment:current`. Its browser portion passed, but the
required deployment command failed:

```text
Error: deployment topology must set minReplicas and maxReplicas to 1
```

Thus the claims contract is release-blocking even though all locally executable
claim assertions pass.

### First read and one-click demo: PASS on the healthy fallback build

A cold live page plainly says it **“See[s] every reminder outcome”**, names
**independent clinics** needing delivery proof and a next step when reminders
fail, and presents **Try it with sample data** with the adjacent explanation
“Opens a sample clinic. Nothing touches real clinic data.” One click opened
`/demo`, showed “Today’s sample reminders,” and displayed the persistent demo
banner with Reset demo and Start for real. This evidence concerns build
`2ec97d2…`, not the candidate.

## Local candidate verification

| Check | Result | Evidence |
| --- | --- | --- |
| Clean install | PASS | `npm ci`: 87 packages; audit reported 0 vulnerabilities. |
| Full suite | PASS | `npm test`: 16 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type / format / lint | PASS | `npm run check`: Svelte 0 errors/warnings, rustfmt, Clippy with warnings denied. |
| Production build | PASS | `npm run build` produced `dist/` and `target/release/reminder-proof-api`. |
| Bundle budgets | PASS | Initial entry JS 28,358 bytes gzip; lazy CIAM chunk 67,778 bytes gzip; CSS 5,535 bytes gzip; emitted fonts 86,060 bytes. |
| No-config service start | PASS | Release binary started with only `PORT=18081`, generated its local durable key, returned `/health`, and completed 100 concurrent health requests. |
| Container image build | NOT RUN | Docker is unavailable in this verifier environment. |

This is a web service, not a library/CLI or PWA; consumer-package and
service-worker checks do not apply.

## Live fallback-build checks

- Cold landing and landing-to-demo request logs contained only
  `clinic-reminder-proof.sociobot.in` HTML, JS, CSS, API, and self-hosted font
  requests. There were no console/page errors or third-party runtime requests.
- Playwright Axe found zero serious or critical findings on `/`, `/demo`,
  `/privacy`, `/terms`, and `/404`. Each had one `h1` and one `main`.
- Factory `verify-url.sh` also passed against the cold live landing: HTTP 200,
  title, `lang=en`, one `h1`, `main`, zero missing image alts, zero unnamed
  buttons, and zero console/page errors (746 ms cold-load measurement).
- At 390 px, reduced-motion emulation, and keyboard-only navigation, all 20
  visible demo controls met 44 px minimum dimensions. First Tab focused the
  skip link with a visible `rgb(0, 95, 204) solid 3px` ring. Reduced-motion
  CSS reduced animation and transition duration to `0.00001s`.
- Response headers included CSP with response-header `frame-ancestors 'none'`,
  HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy,
  permissions policy, and COOP. The hashed JS asset used
  `Cache-Control: public, max-age=31536000, immutable`; HTML was `no-cache`.
- The public service exposed only the required Sociobot Entra authority in its
  configured auth path. The actual login flow was not completed because this
  verifier has no clinic identity.
- Demo-creation limiting was enforced. Across one continuous public-client
  probe (including three preceding demo creations), two further creates were
  `200` and the next was `429`; rejected responses had `Retry-After: 3580`.
  This is consistent with the documented five creations per client per hour.
  The deployment verifier could not complete its fresh six-request probe
  because it fails first on topology.

## Deployment evidence and blockers

`GET /health` returned the fallback identity, not the candidate:

```json
{"status":"ok","build_sha":"2ec97d29b07279e15efb5e82caf002ffe63765e1"}
```

The landing footer likewise rendered `Build 2ec97d2`.

Read-only Azure inspection found:

| Revision | Image | Health / traffic | Topology |
| --- | --- | --- | --- |
| `sf-clinic-reminder-proof--0000050` | full `2ec97d29b07279e15efb5e82caf002ffe63765e1` | Healthy / 0% | exactly one replica; `/durable` and `/backups` Azure Files mounts |
| `sf-clinic-reminder-proof--0000051` | `3a341b7d34f6` | **Unhealthy / 100%** | `minReplicas: 1`, **`maxReplicas: 3`**, no volumes or mounts |

The candidate revision log records the direct startup failure:

```text
initialize durable clinic store: "required durable storage mounts are missing:
/durable, /backups; refusing unsafe production storage"
```

## Defects by severity

| Severity | ID | Finding |
| --- | --- | --- |
| Critical | QA17-01 | Candidate revision `0000051` is unhealthy at declared 100% traffic because it permits three replicas and lacks the required durable/backup Azure Files mounts. This breaks the single-owner persistence and rate-limit boundary. |
| Critical | QA17-02 | The public URL serves fallback build `2ec97d2…`, not requested candidate `3a341b7…`; public behavior cannot accept the candidate. |
| High / Medium / Low | — | No additional independently reproducible defect on the healthy fallback build. |

## Required remediation and recheck

Deploy the exact full candidate image through `npm run deploy:container` so
`deployment/containerapp.json` is composed into the same revision. Require the
candidate to be healthy and the sole 100% traffic target with
`minReplicas=maxReplicas=1` and both `/durable` and `/backups` mounts. Then
confirm `/health` and the footer report the full
`3a341b7d34f6e734791ec37596295cda193374ed` SHA and rerun:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```
