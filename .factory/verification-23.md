# Verify appointment reminder outcomes — verification 23

Date: 2026-09-05 UTC

Work order: `clinic-reminder-proof-verify-23`

Live URL: <https://clinic-reminder-proof.sociobot.in>

Current milestone: M2 — accounts, durable clinic data, and subscriptions

Implementation reviewed: `b58c17feaac9b3ffd9e6fc555036641066cd09ae`

Documentation reviewed: `533c2cff265f4d3f063d0ac968a94c6669dba3fe`

## Verdict

**FAIL — do not accept this release.** There are two findings: one critical
external checkout blocker and one high-severity deployment identity defect.
There are zero untested declared claims.

The reminder workflow itself is strong. Fresh desktop and phone sessions,
the complete sample lifecycle, local product gates, accessibility, security,
tenant fixtures, recovery fixtures, and the live rate-limit recovery state all
passed. A mandatory declared claim nevertheless failed because production is
running a short-tagged image. The live service also defaults to the pilot
billing catalog, where every offered tier is unavailable.

| Severity | Count |
| --- | ---: |
| Critical | 1 |
| High | 1 |
| Medium | 0 |
| Low | 0 |
| Untested claims | 0 |

## Findings

### QA23-01 — High — the active image uses a short mutable tag

`npm run verify:deployment:current` failed with `container image tag must be a
full 40-character Git commit SHA`. Fresh Azure inspection of only this product
found active revision `sf-clinic-reminder-proof--0000068` at 100% traffic with
image `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:533c2cff265f`.

The revision is healthy, runs one ready replica with zero restarts, uses Single
mode, has `minReplicas: 1` and `maxReplicas: 1`, and mounts the product data and
backup shares at `/data` and `/backups`. The defect is deployment identity and
immutability, not the repaired storage topology.

Live `/health` reports the full documentation SHA `533c2cff…`, not the assigned
implementation SHA. The source diff from `b58c17fe…` to `533c2cff…` changes
only `.factory/handoff.md`. After normalizing the embedded build value, the
live 96,535-byte application bundle is byte-for-byte identical to the exact
implementation build. This proves product-code equivalence but does not make
the short image tag satisfy the declared deployment claim.

Required action: deploy the unchanged implementation with a full 40-character
immutable tag through the repository deployment command, then rerun
`npm run verify:deployment:current` after the public allowance window resets.

### QA23-02 — Critical external dependency — pilot checkout is unavailable

The running product has only `PORT` configured, so its documented default is
the Sociobot pilot billing base. Fresh public GETs for Clinic, Practice, and
Network checkout on the pilot host each returned HTTP 404. A signed-in clinic
therefore cannot finish the M2 purchase needed to activate live dispatch.

The production Sociobot gateway now returns HTTP 303 to the approved hosted
checkout for all three tiers. That is an improvement over verification 22's
500 response, but it does not repair the live product's pilot dependency. No
payment was submitted and no hosted session was followed.

This remains external to this repository and was not changed. The operator
must enable all three pilot tiers or move the product to the production catalog
under the planned release process, then verify purchase return, cancellation,
and revocation with an authorized test clinic.

## First screen and sample workflow

Fresh 1440×900 and 390×844 Chromium contexts showed these items before any
scroll:

- Job: `See every reminder outcome.`
- Audience: independent clinics that need delivery proof and a clear next step
  after a reminder fails.
- First action: `Try it with sample data`.
- Result beside the action: `Opens a sample clinic. Nothing touches real clinic
  data.`
- Three facts: sample-only demo actions, no clinical notes in reminders, and
  `$79 per location each month`.

Both contexts opened the populated Northline Sample Clinic with five fictional
appointments. The persistent banner said `Demo — sample data, nothing is saved
to your clinic` and exposed `Reset demo` and `Start for real`. The sample showed
delivery, fallback, consent blocking, a patient reply, and cancellation.

In each context I advanced the reminders, assigned Sofia R. to Sam Rivera,
resolved the exception as `Called patient`, reloaded to prove persistence, and
reset. Reset restored Due 4, Delivered 1, Exceptions 1, cleared the owner, and
moved focus to the demo heading. No `real:` browser key appeared. All recorded
runtime requests stayed on the product origin. No real clinic data or provider
was contacted.

After the public allowance was exhausted, a new desktop click showed the
expected recovery state: `Too many demo actions. Wait, then try again.` with a
`Try again` action. The response was 429 and included `Retry-After: 3549`.
Chromium also logged its generic failed-resource message for that deliberate
429; normal desktop and phone flows had no console errors.

## Claims and clean-checkout gates

A new clone of the named GitHub repository was checked out at documentation SHA
`533c2cff…`. Its only change from implementation SHA `b58c17fe…` is
`.factory/handoff.md`. `npm ci` installed 87 packages with zero reported
vulnerabilities. `npm audit --omit=dev` also reported zero vulnerabilities.

Every one of the 37 commands in `.factory/claims.json` was run exactly as
declared. Thirty-six passed. `single-replica-durable-topology` failed only in
its required live deployment verifier as described in QA23-01; its checked-in
topology test passed. No claim was skipped.

| Claim | Result |
| --- | --- |
| `demo-isolation` | Pass |
| `sample-outcome-coverage` | Pass |
| `consent-channel-guard` | Pass |
| `fallback-order` | Pass |
| `delivery-timeline` | Pass |
| `exception-ownership` | Pass |
| `sample-exception-visibility` | Pass |
| `demo-reset` | Pass |
| `minimal-reminder-content` | Pass |
| `public-price` | Pass |
| `demo-cookie-lifetime` | Pass |
| `demo-replica-continuity` | Pass |
| `no-tracking` | Pass |
| `explicit-theme-choice` | Pass |
| `request-protection` | Pass |
| `rate-limit-policy` | Pass locally; live 429 and `Retry-After` observed |
| `security-headers` | Pass |
| `build-identity` | Pass for the running `533c2cff…` build |
| `managed-auth-storage` | Pass |
| `signed-calendar-intake` | Pass |
| `approved-whatsapp-dispatch` | Pass |
| `twilio-receipt-verification` | Pass |
| `resend-receipt-verification` | Pass |
| `managed-secret-encryption` | Pass |
| `managed-data-minimisation` | Pass |
| `no-marketing-campaigns` | Pass |
| `signed-in-export-delete` | Pass |
| `managed-provider-fallback-receipt` | Pass |
| `managed-billing-return` | Fixture pass; live pilot dependency is QA23-02 |
| `managed-storage-recovery` | Pass |
| `single-replica-durable-topology` | **Fail** — short active image tag |
| `ciam-sign-in` | Pass |
| `tenant-isolation` | Pass |
| `durable-onboarding` | Pass |
| `subscription-price` | Pass |
| `data-export` | Pass |
| `account-deletion` | Pass |

Additional clean-checkout results:

- `npm test`: pass — 21 Vitest, 42 Rust, and 47 Chromium tests.
- `npm run check`: pass — zero Svelte diagnostics; rustfmt and Clippy clean.
- Exact-implementation `npm run build`: pass — `dist/` and the release API
  binary produced; initial app JavaScript 31.82 kB gzip, CSS 5.79 kB gzip.
- Default runtime with only `PORT`: pass; `/health` reported `b58c17fe…` and
  100 concurrent health requests returned 100 HTTP 200 responses.
- Restart persistence, encrypted recovery pairing, transient SQLite lock
  waiting, tenant isolation, role boundaries, migration reversal, seven-day
  deletion recovery, and 30-day backup retention passed their fresh temporary
  database tests.

## Live routes, accessibility, privacy, and recovery

- `/opt/fleet/lib/verify-url.sh`: pass; HTTP 200, title, `lang=en`, one H1, a
  main landmark, no missing alt text, no unnamed button, and no console error.
- Playwright axe in light and dark treatments: zero violations on `/`, `/demo`,
  `/start`, `/app`, `/privacy`, `/terms`, and an unknown route.
- Mobile Lighthouse: 100 Performance, 100 Accessibility, 100 Best Practices,
  and 100 SEO; LCP 1.41 s, CLS 0.0007, TBT 31.5 ms.
- Every inspected page had one H1 and one main landmark. All M2 onboarding,
  staff, billing, data-control, sign-in, callback, legal, app, and demo routes
  had specific titles and no 390 px overflow.
- At 200% text in dark mode, every public and legal route reflowed without
  horizontal loss. No visible demo link, button, or select was below 44×44 px.
- Keyboard Tab exposed the skip link; Enter focused `main`. Resolving an
  exception focused `Undo resolution`. Reduced-motion media was honored and
  settled with no active animation.
- Offline mode kept the loaded ledger readable, showed its explicit read-only
  notice, and disabled advance and resolution. No offline-reload or installable
  PWA claim is made, so update-install checks do not apply.
- `/privacy` and `/terms` returned 200. Anonymous export and deletion requests
  returned structured 401 responses with `WWW-Authenticate: Bearer` and unique
  matching request IDs.
- Malformed JSON, wrong content type, a 17 KB body, and missing authentication
  returned 400, 415, 413, and 401 respectively, with plain recovery messages
  and matching request IDs.
- The CIAM button redirected to the documented Sociobot customer tenant using
  authorization code flow, the correct client and callback, and PKCE. No user
  credential was entered.
- An unknown route returned the designed HTTP 404 with title
  `Page not found — Reminder Proof`, H1 `Page not found`, and a route home. Its
  generic browser 404 console entry is expected and is not a defect.
- `robots.txt`, `sitemap.xml`, favicon, 180 px touch icon, 1200×630 social card,
  internal links, and the Param Factory link resolved. Security headers and
  immutable hashed-asset caching matched the documented policy.

## Earlier finding disposition

Every earlier review, polish, and verification report was inspected.

| Earlier findings | Current disposition and proof |
| --- | --- |
| Review F-1-1 through F-1-8 | Fixed. Descriptive headings, declared exception visibility, short README sentences, and consistent terms remain. Copy tests and the live first-read check passed. |
| Review F-2-1 through F-2-5 | Fixed. Signed intake, approved WhatsApp, callback signatures/replay protection, secret encryption, and clinical-field rejection each have declared passing claims. |
| Review F-3-1 through F-3-8 | Fixed. The untestable originality phrase is absent; the footer build, direct 404 heading, demo wording, same-site wording, duplicate-intake wording, and messaging-provider terminology all passed. |
| Initial verification: lost demo workspace and contradictory demo/real scope | Fixed. Cookie continuity, reset, navigation, reload, and server-instance regressions passed; the managed clinic path is present and protected. |
| Initial/QA3–QA7 rate-limit, request-protection, missing-request-ID, and spoofing findings | Fixed in product behavior. Live 429 included `Retry-After`; varied caller prefixes no longer create buckets; 400/401/413/415 errors carry matching UUID request IDs. |
| QA3–QA7 incomplete or structurally invalid claim coverage | Fixed. All 37 declared commands were exercised; tags exist for the managed fallback and billing return. There are no unlisted current landing or README claims. |
| Verification 5 mobile 200% loss and focus loss; QA7 theme/meta defects; QA11-02 touch targets | Fixed. Live 200% reflow, explicit themes, one description meta, 44 px controls, and post-resolution focus all passed. |
| QA3 key permissions and QA5/QA6/QA11–QA22 replica/storage findings | The data-safety part is fixed: one running replica, `/data` and `/backups`, synchronous recovery pairs, owner-only key tests, and restart/lock regressions passed. Image identity is not fully fixed because QA23-01 reintroduces a short tag. |
| QA12–QA20 unhealthy/stale/candidate-mismatch deployments | The active revision is now healthy, ready, and at 100% with matching health/footer identity. The remaining full-tag defect is QA23-01. |
| QA15-04 stale hard-coded deployment verifier | Fixed. The verifier derives the current health identity; it correctly rejected the short active image. |
| QA18-02 reused live rate-test identity | The helper now randomizes its declared client, and local regressions pass. Production correctly ignores caller prefixes, so repeated public runs still share the ingress-captured allowance by design. |
| Verification 8 deliberate 429 console noise | Expected recovery behavior, not a normal-flow defect. Fresh normal flows were clean; the deliberate 429 showed a plain recovery action. |
| Checkout findings from verifications 3, 4, and 22 | Still blocks M2 through the pilot catalog, now recorded separately as QA23-02. Production checkout redirects work; pilot checkout returns 404. |

## Scope and future work

This verifies the current M2 public promises. M3–M5 implementation notes in the
venture plan were not treated as new acceptance demands, and planned M6 growth
work was not presented as shipped. No AI feature is justified for this
deterministic consent and evidence workflow. The product is a web service, not
a CLI, library, or desktop artifact, so clean consumer-install checks do not
apply.

No product code, deployment, data, billing setting, credential, or external
service was changed during this verification.
