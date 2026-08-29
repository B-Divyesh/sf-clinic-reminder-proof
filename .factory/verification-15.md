# Independent product verification 15 — FAIL

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-15`

Requested candidate: `ae20862641e90b0a265fc75ab76e5273159e7bef`

Reachable clone commit: `ae2086a2e672b9c61f3ea2867d16a2158ec8d94a`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release and do not accept real clinic data.** The exact requested candidate is absent from both the supplied clone and the named GitHub remote, so it cannot be built or matched to production. Fresh Azure evidence also shows an unhealthy, short-tagged latest revision with unsafe replica/storage topology declared at 100% traffic. Public requests fall back to the older `e831167…` build. The mandatory claims gate fails on that deployed topology.

The currently served older build is otherwise healthy in functional, accessibility, privacy, security-header, and performance checks. Those results do not establish the requested candidate.

## Mandatory first checks

### First-read and one-click demo: PASS on the live older build

A cold 1440×900 visit returned HTTP 200 with no console/page error. Its first screen answers all three required questions in plain words:

- What it does: **“See every reminder outcome.”**
- For whom: **“For independent clinics that need delivery proof and a clear next step when reminders fail.”**
- What to do first: **“Try it with sample data”**, beside **“Opens a sample clinic. Nothing touches real clinic data.”**

The action opens `/demo` in one click. The resulting screen is already populated with five realistic fictional appointments and keeps the banner **“Demo — sample data, nothing is saved to your clinic”**, plus **Reset demo** and **Start for real**.

### Claims gate: FAIL (30/31 manifest entries passed)

`.factory/claims.json` exists with 31 entries. After `npm ci`, every literal `test` command was run separately in manifest order. The first 30 passed. The final composite claim failed, which is release-blocking by contract.

| Claim ID | Result | Evidence |
| --- | --- | --- |
| `demo-isolation` | PASS | Exact manifest Playwright command passed. |
| `sample-outcome-coverage` | PASS | Exact manifest Playwright command passed. |
| `consent-channel-guard` | PASS | Exact manifest Playwright command passed. |
| `fallback-order` | PASS | Exact manifest Playwright command passed. |
| `delivery-timeline` | PASS | Exact manifest Playwright command passed. |
| `exception-ownership` | PASS | Exact manifest Playwright command passed. |
| `sample-exception-visibility` | PASS | Exact manifest Playwright command passed. |
| `demo-reset` | PASS | Exact manifest Playwright command passed. |
| `minimal-reminder-content` | PASS | Exact manifest Playwright command passed. |
| `public-price` | PASS | Exact manifest Playwright command passed. |
| `demo-cookie-lifetime` | PASS | Exact manifest Playwright command passed. |
| `demo-replica-continuity` | PASS | Exact manifest Playwright command passed. |
| `no-tracking` | PASS | Exact manifest Playwright command passed. |
| `explicit-theme-choice` | PASS | Exact manifest Playwright command passed. |
| `request-protection` | PASS | Exact manifest Playwright command passed. |
| `rate-limit-policy` | PASS | Exact manifest Playwright command passed. |
| `security-headers` | PASS | Exact manifest Playwright command passed. |
| `build-identity` | PASS | Local exact manifest Playwright command passed. Live identity is stale; see QA15-02. |
| `managed-auth-storage` | PASS | Exact Rust test and Playwright command both passed. |
| `signed-calendar-intake` | PASS | Exact manifest Playwright command passed. |
| `approved-whatsapp-dispatch` | PASS | Exact manifest Playwright command passed. |
| `twilio-receipt-verification` | PASS | Exact manifest Playwright command passed. |
| `resend-receipt-verification` | PASS | Exact manifest Playwright command passed. |
| `managed-secret-encryption` | PASS | Exact manifest Playwright command passed. |
| `managed-data-minimisation` | PASS | Exact manifest Playwright command passed. |
| `no-marketing-campaigns` | PASS | Exact Rust test and Playwright command both passed. |
| `signed-in-export-delete` | PASS | Exact Rust test and Playwright command both passed. |
| `managed-provider-fallback-receipt` | PASS | Exact manifest Playwright command passed. |
| `managed-billing-return` | PASS | Exact manifest Playwright command passed. |
| `managed-storage-recovery` | PASS | Exact manifest Playwright command passed. |
| `single-replica-durable-topology` | **FAIL** | Source-template assertion passed; required deployment verifier failed: `deployment topology must set minReplicas and maxReplicas to 1`. |

The final manifest command also hard-codes the previous `e831167…` identity rather than the requested candidate. Landing and README claim-like copy was cross-checked against the manifest and `.factory/copy-audit.md`; no additional unlisted public promise was found in the reachable source.

## Source and candidate identity

The supplied clone started at the work order base, not the requested candidate:

```text
$ git rev-parse HEAD
ae2086a2e672b9c61f3ea2867d16a2158ec8d94a

$ git fetch origin ae20862641e90b0a265fc75ab76e5273159e7bef
fatal: remote error: upload-pack: not our ref ae20862641e90b0a265fc75ab76e5273159e7bef
```

After a full refresh, `origin/main` and remote `HEAD` remained `ae2086a2e672b9c61f3ea2867d16a2158ec8d94a`; `git ls-remote origin` contained no requested SHA. All local results below therefore describe that reachable base, not the unavailable candidate. The base differs from the serving `e831167…` source only in `.factory/claims.json` and `.factory/handoff.md`; this does not prove what the unavailable candidate contained.

## Clean local gates on reachable commit

| Check | Result | Evidence |
| --- | --- | --- |
| Locked install | PASS | `npm ci`: 87 packages, 0 audit vulnerabilities. |
| Complete tests | PASS | `npm test`: 14 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type/format/lint | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt passed; Clippy passed with warnings denied. |
| Production build | PASS | `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. |
| Runtime default contract | PASS | Release binary started with only `PORT=18081` among app settings, generated a 0600 key/database, served health/metrics, logged generated configuration, and shut down cleanly. |
| Container build | NOT RUN | Docker is unavailable. No candidate image can be built because the candidate tree is unavailable. |

Local production assets were 28.63 KB gzip entry JS, 68.23 KB gzip lazy MSAL chunk, 5.54 KB gzip CSS, and 85.96 KB total font files. These are within the recorded budgets. `dist/` exists.

## End-to-end behavior and recovery

The full suite was rerun against the live origin: **40/40 passed**. Independently, a fresh browser advanced due reminders, assigned Sofia R. to Sam Rivera, resolved the exception, reloaded to confirm persistence, and used **Undo resolution**. The open task and controls recovered correctly.

| Case | Observed result |
| --- | --- |
| Malformed JSON | 400 `json_invalid`; response/request IDs matched. |
| Wrong media type | 415 `content_type_invalid`; response/request IDs matched. |
| Invalid demo owner | 422 `owner_invalid`; state stayed unchanged, then a valid owner recovered. |
| Unknown reminder | 404 `reminder_missing`; response/request IDs matched. |
| Anonymous clinic/billing access | 401 `bearer_required`, `WWW-Authenticate: Bearer`, correlatable request ID. |
| 16,383-byte JSON write | 200. |
| 16,384-byte JSON write | 200. |
| 16,385-byte JSON write | 413 `body_too_large`. |
| Demo reload/reset | State persisted on reload; reset restored the original seed. |
| 100 concurrent `/health` reads | 100×200 in 490 ms (203.9 requests/second observed). |

The demo exercised consent guard, SMS/WhatsApp/email fallback, provider timeline, staff-owned exception, resolution, undo, and reset. Provider, signed-intake, receipt, encryption, tenant isolation, export/delete, recovery, and billing behavior additionally passed fixture integration tests. No real patient, provider, payment, or clinic identity was used.

## Rate limiting

- Demo creation allowance: **five successful requests per client per hour**. Request six returned 429, `rate_limited`, and `Retry-After: 3599`.
- A burst of 60 protected checkout requests produced 43×401 and 17×429; every 429 had `Retry-After: 1`. The configured general limiter is burst 40 with one token every 50 ms, so tokens replenished during the burst.
- `/health` is intentionally exempt. `/metrics` and all API routers carry the general limiter.

This proves 429/`Retry-After` on the serving revision. The unsafe three-replica latest template would not preserve one process-local limiter owner if it became healthy.

## Privacy, authentication, and response policy

- The manual landing-to-demo core flow made 16 requests, all same-origin. No tracker, CDN font, messaging provider, billing service, or other runtime third party appeared. Console/page errors: zero.
- Demo cookie: `Path=/api/v1/demo`, `HttpOnly`, `Secure`, `SameSite=Lax`, `Max-Age=86400`.
- Auth exposes only tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client `25c704f4-465a-47af-80ab-2c489466b697`, and `sociobotcustomers.ciamlogin.com` authority.
- **Sign in with Microsoft** navigated to that exact authority using code + PKCE (`S256`), the expected callback, and documented scopes. CIAM offered **Create one**. No alternate auth provider was used.
- Responses include HSTS, `nosniff`, strict-origin referrer policy, permissions policy, COOP, and CSP with `frame-ancestors 'none'`.
- Hashed JS uses gzip and `public, max-age=31536000, immutable`; HTML uses `no-cache`.

Production sign-up and a real checkout were not completed because no clinic identity or payment authority was supplied. The fixture managed-billing claim passed and source uses the Sociobot billing API.

## Accessibility, responsive behavior, and performance

- Factory `verify-url.sh` passed: 200, title, `lang=en`, one H1, main, alt/name checks, and zero console/page errors; cold load 711 ms.
- Standalone axe-core 4.11.4 found **0 violations** on `/`, `/demo`, `/privacy`, and `/terms`. Playwright Axe found zero violations in light and dark themes.
- At 390×844 with reduced motion and dark theme, `scrollWidth` was 390 and motion duration effectively zero. Public and managed routes reflowed at 200% text.
- Keyboard smoke passed: first Tab focused the skip link with a visible 3 px `#005fcc` outline; Enter focused `<main>`. Resolution focused **Undo resolution**. Touch targets passed 44 px.
- Titles, one-H1 structure, landmarks, links, deep links, back navigation, styled HTTP 404, offline-read state, and disabled offline writes passed.
- Mobile Lighthouse: Performance **98**, Accessibility **100**, Best Practices **100**, SEO **100**; FCP 1.388 s, LCP 1.498 s, TBT 169.5 ms, CLS 0.000742.
- Live first load transferred 28.6 KB JS and 55.98 KB self-hosted WOFF2 fonts; local CSS was 5.54 KB gzip. Budgets pass.

This is not a library/CLI and has no service worker or web app manifest, so consumer packing and PWA update/offline-reload checks do not apply. Its supported read-only offline UI state passed.

## Live deployment evidence

Fresh Azure inspection found:

```json
{
  "latestRevision": "sf-clinic-reminder-proof--0000047",
  "latestReadyRevision": "sf-clinic-reminder-proof--0000046",
  "latestTemplate": {
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:ae2086a2e672",
    "minReplicas": 1,
    "maxReplicas": 3,
    "mounts": null,
    "volumes": null
  },
  "declaredTrafficRevision": {
    "name": "sf-clinic-reminder-proof--0000047",
    "health": "Unhealthy",
    "trafficWeight": 100,
    "replicas": 1
  },
  "actualHealthyRevision": {
    "name": "sf-clinic-reminder-proof--0000046",
    "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:e8311677822d4a60183b9efcd5aab8980fc2b200",
    "health": "Healthy",
    "maxReplicas": 1,
    "mounts": ["/durable", "/backups"]
  }
}
```

Public `/health` returns `e8311677822d4a60183b9efcd5aab8980fc2b200`; footers on all tested routes display `Build e831167`. Public traffic matches neither requested candidate `ae208626…` nor the short `ae2086a2e672` unhealthy image.

## Defects by severity

### QA15-01 — Critical — requested candidate cannot be obtained or tested

The SHA is missing from the clean clone and the named remote rejects it as “not our ref.” Source review, candidate build, and deployed-source matching are impossible.

### QA15-02 — Critical — live public build does not match the requested candidate

Health/footer identify `e831167…`; Azure's newest image is only a short `ae2086a2e672` tag and is unhealthy. Neither is `ae20862641e90b0a265fc75ab76e5273159e7bef`.

### QA15-03 — Critical — declared traffic revision has unsafe durability/scaling topology

Revision `0000047` has `maxReplicas: 3` and no durable/backup mount or volume, yet is declared the 100% traffic target. If made healthy, multiple SQLite/rate-limit owners and ephemeral keys/storage could split or lose clinic state. This fails `single-replica-durable-topology`.

### QA15-04 — High — claims manifest deployment command is pinned to an old build

The required command hard-codes `EXPECTED_BUILD_SHA=e831167…`. It cannot establish the requested candidate identity and would reject a correctly deployed later build unless edited.

No other reproducible High, Medium, or Low defects were found on the older serving build.

## Required remediation

1. Push the exact full candidate to the named remote.
2. Build/tag it with its full 40-character SHA.
3. Deploy through the checked-in topology so one healthy revision has 100% traffic, `minReplicas=maxReplicas=1`, and both Azure Files mounts.
4. Make health and footer report that exact SHA.
5. Make the deployment claim receive/derive the candidate SHA rather than pinning `e831167…`.
6. Rerun all 31 claims and `EXPECTED_BUILD_SHA=<exact-candidate> npm run verify:deployment` from a clean clone.
