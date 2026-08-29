# Independent product verification 5 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-5`
Candidate: `454c4211fe1ffe5ecc9116b602eaae5e3d080002`
Live URL: https://clinic-reminder-proof.sociobot.in

## Verdict

**FAIL — do not release or accept real clinic data.** The earlier checkout-only
failure is repaired: the live service is the exact candidate and the production
Sociobot endpoint opens a real $79/month hosted checkout. The public demo,
privacy behavior, auth boundary, normal responsive UI, local suite, production
build, and performance checks pass.

Release is nevertheless blocked by fresh live evidence that the active
deployment has three replicas and no durable or backup volumes. Managed clinic
SQLite state is therefore replica-local and ephemeral, contrary to the
product's durability claim. The same topology triples process-local request
allowances. A mandatory claim command also failed on its first clean run, and
several public routes fail the required 200% text-reflow check.

## First-read gate — PASS

Cold desktop and 390 px mobile reads answered all three required questions in
the initial viewport:

- **What it does:** “See every reminder outcome.”
- **For whom:** “For independent clinics that need delivery proof and a clear
  next step when reminders fail.”
- **What to click first:** **“Try it with sample data”**, next to “Opens a
  sample clinic. Nothing touches real clinic data.”

The action opens `/demo` in one click and immediately shows five realistic
sample appointments, evidence, and an exception queue. Evidence:
`qa-evidence/cold-desktop.png` and `qa-evidence/cold-mobile.png`.

## Release-blocking findings

### Critical — live managed clinic storage is neither shared nor durable

The repository contract requires exactly one replica with Azure Files mounted
at `/durable` and `/backups`. Fresh Azure control-plane evidence for active
revision `sf-clinic-reminder-proof--0000023` instead showed:

```text
image: sociobotregistry.azurecr.io/sf-clinic-reminder-proof:454c4211fe1f
minReplicas: 1
maxReplicas: 3
active replicas: 3 (RunningAtMaxScale)
template volumes: null
container volumeMounts: null
configured environment: PORT only
```

The release image defaults its SQLite writer to `/data`; that path is also not
mounted. A clinic can therefore create or update a workspace on one replica,
read a different workspace state on another, and lose all managed records when
a replica is replaced. `/durable` and `/backups` are only local container
directories without mounts. This directly contradicts the declared
`managed-auth-storage` claim, README persistence promises, checked-in
`deployment/containerapp.json`, and `.factory/operations.md`.

Required fix: deploy the candidate with one replica and both ReadWrite mounts,
then prove a signed-in tenant survives a replica replacement and backup restore.
Do not accept clinic records before that proof. Longer term, move clinic data
and rate state to shared services before scaling above one replica.

### Critical — live per-client limits multiply across replicas

The code holds both limiters in process memory. Fresh production probes used
one stable first `X-Forwarded-For` hop and varied only later proxy hops:

- Sequential demo creation on one connection returned
  `200, 200, 200, 200, 200, 429`; request six included
  `Retry-After: 3599`.
- **18 simultaneous requests from the same client returned 15 × 200 and only
  3 × 429 in 162 ms.** The documented allowance is five creations per client
  per hour.
- A 120-request `/metrics` burst returned 46 × 200 and 74 × 429 in 612 ms.
- A 120-request unauthenticated `/api/v1/billing/checkout` burst returned
  **92 × 401 and only 28 × 429** in 600 ms; rejected requests included
  `Retry-After: 1`.

The 15 successful creations match three replicas each admitting five. A
single client can go beyond the documented allowance, so the server-side rate
limit acceptance requirement and the live `rate-limit-policy` claim fail.

Required fix: enforce limits at ingress or in a shared store, or restore the
documented single-replica topology. Add a multi-replica production regression;
a local single-process claim test cannot prove service-wide enforcement.

### High — one mandatory claim command fails from the clean checkout

After locked `npm ci`, every command in `.factory/claims.json` was invoked
individually before general QA. The first exact command failed:

```text
npm run test:e2e -- --grep @claim:demo-isolation
Error: Timed out waiting 120000ms from config.webServer.
```

On the clean cache, Playwright spent the full allowance compiling the Rust
backend and never started the claim. The next command completed the remaining
compile, and every subsequent exact command passed. The full warm-cache
`npm test` later passed, including `demo-isolation`, but the acceptance contract
states that any failing listed claim command is release-blocking. The test
harness needs a cold-build-safe server timeout or a separate build step.

### High — 200% text causes horizontal loss on four public routes

At a 390 px viewport with root text resized to 200%, direct measurements were:

| Route | Viewport width | Document width | Excess |
| --- | ---: | ---: | ---: |
| `/` | 390 px | 416 px | 26 px |
| `/demo` | 390 px | 452 px | 62 px |
| `/privacy` | 390 px | 430 px | 40 px |
| `/terms` | 390 px | 441 px | 51 px |

The demo `<h1>` extended to 451.64 px. `/start`, `/app`, and the 404 route did
reflow to 390 px. Normal-size 390 px rendering is good, and axe cannot detect
this issue, but the attached accessibility contract requires 200% text without
loss. Evidence: `qa-evidence/live-demo-mobile-dark-200.png`.

### High — claim manifest omits or does not prove public promises

Each of the 19 listed IDs has exactly one matching `@claim:<id>` tag. However,
the README makes additional measurable promises absent from `claims.json`,
including synchronous durable checkpoints, daily recovery pairs, 30-day
retention, startup restoration, a production single-replica topology, and
mounted Azure Files shares. The last two are false in production.

The `public-price` test only checks that the phrase “plus published messaging
charges” is repeated on Landing and Terms. No messaging rate or link to a rate
schedule exists on those pages, in the README, or in hosted checkout. The test
therefore does not prove that those charges are published. Under the claims
contract, unlisted or merely text-matched promises are release-blocking.

### Medium — keyboard focus is lost after resolving an exception

The complete demo can be operated with Tab, arrows, and Enter. After activating
“Resolve as Called patient,” however, the focused button is removed and focus
falls to `<body>` instead of moving to the new “Undo resolution” action or the
status notice. This interrupts a keyboard user's position in a long ledger.

## Claims matrix — clean checkout, exact commands

`npm ci` installed 87 packages with zero reported vulnerabilities. All commands
were run exactly as declared and in manifest order.

| Claim ID | First exact command result |
| --- | --- |
| `demo-isolation` | **FAIL** — Playwright web server timed out at 120 seconds during cold Rust compilation |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS command; claim-quality defect above |
| `demo-cookie-lifetime` | PASS |
| `demo-replica-continuity` | PASS |
| `no-tracking` | PASS |
| `request-protection` | PASS |
| `rate-limit-policy` | PASS locally; **FAIL live service-wide** |
| `security-headers` | PASS |
| `build-identity` | PASS |
| `managed-auth-storage` | PASS local fixtures; **FAIL live durability** |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS fixture; production checkout independently reached |

## Local gates

| Check | Result |
| --- | --- |
| Candidate/worktree before QA | PASS — clean, exact `454c4211fe1ffe5ecc9116b602eaae5e3d080002` |
| `npm ci` | PASS — 87 packages, 0 vulnerabilities |
| `npm test` after cold compile | PASS — 6 Vitest, 23 Rust, 24 Chromium |
| `npm run check` | PASS — Svelte 0 errors/warnings, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS — exact web and release API build; `dist/` produced |
| Release binary with only `PORT=18085` | PASS — `/health` returned `dev`; JSON startup logs; generated key/database mode 0600 in a mode-0700 directory |
| Docker image build | Not run — Docker and Podman are absent; the live exact-candidate image and Dockerfile were inspected instead |

Production web output: public entry JS 80,084 bytes raw / 27,900 gzip;
CSS 24,505 / 5,310 gzip. The 271,994-byte MSAL chunk is lazy and did not load
on Landing or Demo.

## End-to-end product evidence

- The live demo starts with five fictional appointments. Once completion was
  awaited, “Advance due reminders” changed delivered evidence from 1 to 3.
- Jordan L.'s evidence showed an approved WhatsApp template rejected with
  `TEMPLATE_REJECTED`, followed by consented email delivery
  `DELIVERED-200`, all marked simulated.
- Sofia R.'s exception was assigned to Sam Rivera, resolved as “Called
  patient,” survived reload, was undone, and reset to the original seed.
- Malformed JSON returned structured `400 json_invalid`; a 17 KB body returned
  structured `413 body_too_large` and an actionable 16 KB limit message.
- The empty signed-out `/app` state gives one next step. Managed storage,
  provider fallback/receipt, tenant isolation, invalid signatures, export,
  deletion, and backup restoration pass local Rust fixtures. A live signed-in
  persistence test was not safe because the deployed storage is ephemeral.
- Keyboard-only Tab/arrow/Enter operation reached and completed assignment and
  resolution. The skip link receives a visible 3 px focus ring. No keyboard
  trap was found; the post-resolution focus defect is recorded above.

## Privacy, auth, billing, headers, and deployment identity

- A full Landing → Demo → assign → resolve → reload → undo → advance → inspect
  → reset flow made only same-origin requests. No analytics, CDN font,
  messaging-provider, billing, or AI request loaded. Normal flows had zero
  console errors, page errors, and failed requests.
- The live `rp_demo` cookie is `HttpOnly`, `Secure`, `SameSite=Lax`, scoped to
  `/api/v1/demo`, and has `Max-Age=86400`.
- Browser responses include CSP with response-header `frame-ancestors 'none'`,
  HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP.
  Hashed assets use `public, max-age=31536000, immutable`; HTML uses `no-cache`.
- `/health` returned the exact candidate SHA. Live JS and CSS SHA-256 values
  exactly match the local production build.
- All public routes have `lang=en`, one `<h1>`, `<main>`, route-specific title,
  canonical/Open Graph/Twitter metadata, and correct 200/404 status. Local link
  crawl passed.
- `/api/v1/auth/config` exposes only the required tenant and SPA client. The
  sign-in button reached `sociobotcustomers.ciamlogin.com` using authorization
  code + PKCE and the required callback. The hosted screen offers account
  creation. Unauthenticated clinic and billing endpoints return 401 with
  `WWW-Authenticate: Bearer`.
- The previous checkout outage is fixed. The production Sociobot product
  endpoint returned HTTP 303, and the hosted checkout showed “Reminder Proof,”
  `$79.00 / Month`, and the one-clinic-location description. No payment was
  submitted.

## Accessibility, responsive behavior, and performance

- Desktop and normal-size 390 px layouts are clear and usable in light and
  dark modes. Tested buttons, selects, and button-styled links are at least
  44 × 44 px. Reduced-motion mode collapses animation/transition durations to
  `0.00001s`.
- Playwright axe found **zero serious/critical findings** on `/`, `/demo`,
  `/start`, `/app`, `/privacy`, `/terms`, and the 404 route in both light and
  dark color schemes. The 200% manual reflow defect remains.
- Lighthouse 12.8.2 mobile: Performance **98**, Accessibility **100**, Best
  Practices **100**, SEO **100**; FCP **1.3 s**, LCP **1.3 s**, CLS **0**,
  TBT **150 ms**, total transfer **58 KiB**.
- This product is not a PWA, library, or CLI, so service-worker/offline reload
  and consumer-package checks do not apply.

## Required path to PASS

1. Redeploy with `maxReplicas=1` and working `/durable` plus `/backups` mounts;
   prove tenant persistence across a replica replacement and a restore.
2. Make rate limits service-wide and prove that every request after five demo
   creations is 429 with `Retry-After`, including concurrent/multi-replica
   traffic. Repeat for a protected endpoint.
3. Make every exact claim command pass from a cold clean checkout.
4. Fix 200% text reflow on Landing, Demo, Privacy, and Terms and preserve focus
   after exception resolution.
5. Add the operational storage/retention promises to `claims.json` with
   observable tests, and publish actual messaging charges or remove the word
   “published.”

No product source code was modified during verification.
