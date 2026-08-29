# Independent product verification 14 — FAIL

Date: 2026-08-29 UTC  
Work order: `clinic-reminder-proof-verify-14`  
Candidate: `e16e61c4c300fe88b9b2705e890127566f89ca28`  
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The clean candidate is locally healthy, but its required production deployment claim fails. Azure reports the candidate image on a traffic-bearing revision that permits three replicas and has neither durable Azure Files mount. Separately, public `/health` and the footer still identify the prior `c2b1ace…` build, not this candidate.

## Required first checks

### First-read and one-click demo: PASS

Cold-load text was plain and complete:

- **What:** “See every reminder outcome.”
- **For whom:** “For independent clinics that need delivery proof and a clear next step when reminders fail.”
- **First action:** visible **Try it with sample data**, with “Opens a sample clinic. Nothing touches real clinic data.”

One click opened `/demo` and showed the persistent “Demo — sample data, nothing is saved” banner with Reset demo and Start for real.

### Claims gate: FAIL (30/31 product claim commands passed)

`.factory/claims.json` is present with 31 entries. After clean `npm ci`, every literal command was invoked separately in manifest order through the shipped demo entry point. The 30 demo/managed product claim tests passed; the complete 40-test Chromium run afterwards also passed. The last composite claim, `single-replica-durable-topology`, passed its browser topology assertion but its required trailing `npm run verify:deployment` failed:

```
Error: deployment topology must set minReplicas and maxReplicas to 1
```

That is release-blocking under the claims contract. The passing claims covered demo isolation and reset, consent, fallback, delivery timeline, staff exception ownership, privacy/minimal content/no tracking, price, security headers, build identity, rate limits, CIAM-protected managed storage, signed intake, provider receipts, encryption/minimisation, billing, recovery, export/delete, and the checked-in topology.

## Local candidate checks

| Check | Result | Evidence |
| --- | --- | --- |
| Clean install | PASS | `npm ci` installed 87 locked packages; audit reported 0 vulnerabilities. |
| Full tests | PASS | `npm test`: 12 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type, format, lint | PASS | `npm run check`: Svelte 0 errors/0 warnings, rustfmt, and Clippy with warnings denied. |
| Production build | PASS | `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. |
| Bundle budgets | PASS | public entry JS 28,191 bytes gzip; CSS 5,553 bytes gzip; WOFF2 total 66,460 bytes. |
| Container build | NOT RUN | Docker is unavailable in this verifier container (`docker: command not found`); the repository production build above passed. |

## Live functional, privacy, accessibility, and response evidence

- A fresh desktop demo created only same-origin page/API/font requests. Advancing due reminders, assigning and resolving the sample exception, and undoing it produced no console errors or page errors. No third-party runtime/tracking request appeared.
- A live Axe scan of the demo reported zero serious or critical violations. At 390 px with dark theme and reduced-motion emulation, the page had no horizontal overflow (`scrollWidth: 390`), no console error, and a visible 3 px focus outline. Resetting focus to body then Tab reached the skip link with its 3 px focus ring.
- Landing, demo, privacy, terms, and 404 each returned one `h1`, `main`, `header`, `nav`, `footer`, `lang=en`, and no image missing `alt`.
- Live response headers include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP. The hashed JS asset uses `Cache-Control: public, max-age=31536000, immutable`.
- `/api/v1/auth/config` names only the Sociobot Entra External ID authority `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`; anonymous `/api/v1/clinic` returned `401` with `WWW-Authenticate: Bearer` and a request ID.
- The observed demo-create allowance is **five** successful `POST /api/v1/demo/workspaces` calls per stable first `X-Forwarded-For` hop. The sixth returned `429`, JSON code `rate_limited`, and `Retry-After: 3599`.

## Release-blocking findings

### QA14-01 — Critical — traffic-bearing candidate revision violates durable single-owner topology

Read-only Azure inspection found:

```json
{
  "latestReadyRevision": "sf-clinic-reminder-proof--0000043",
  "appTemplate": {"minReplicas": 1, "maxReplicas": 3, "mounts": null},
  "candidateTrafficRevision": {
    "name": "sf-clinic-reminder-proof--0000044",
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:e16e61c4c300",
    "trafficWeight": 100,
    "replicas": 1,
    "scale": {"minReplicas": 1, "maxReplicas": 3},
    "mounts": null,
    "volumes": null
  }
}
```

The checked-in contract requires exactly one maximum replica and both `/durable` and `/backups` Azure Files mounts. The deployed candidate instead lacks those boundaries. This can split SQLite/key/recovery state and process-local rate-limit state, and makes managed clinic persistence unsafe.

### QA14-02 — Critical — live request identity is stale and does not match the candidate

At the same time, `GET /health` returned:

```json
{"status":"ok","build_sha":"c2b1aced3ed7e5585d9db4eb73ffff495d1874e0"}
```

and the cold landing footer displayed `Build c2b1ace`. The requested candidate is `e16e61c4c300fe88b9b2705e890127566f89ca28`. Therefore live traffic is not demonstrably serving the candidate, despite Azure describing revision `0000044` as 100% traffic.

## Defects by severity

| Severity | ID | Summary |
| --- | --- | --- |
| Critical | QA14-01 | Candidate traffic revision has `maxReplicas: 3` and no durable/backup Azure Files mounts. |
| Critical | QA14-02 | Public health/footer identify the prior `c2b1ace…` build rather than candidate `e16e61c…`; Azure and runtime identity disagree. |
| High / Medium / Low | — | No additional independently reproducible defects found in this run. |

## Required remediation and recheck

Deploy the exact full candidate image and checked-in `deployment/containerapp.json` in one revision. Confirm that revision alone has 100% traffic, exactly one maximum replica, both Azure Files mounts, and `/health` returns the full `e16e61c4c300fe88b9b2705e890127566f89ca28` identity. Then rerun:

```sh
EXPECTED_BUILD_SHA=e16e61c4c300fe88b9b2705e890127566f89ca28 npm run verify:deployment
```
