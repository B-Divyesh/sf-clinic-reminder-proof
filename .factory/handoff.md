# Repair 6 handoff — verification 7 findings resolved

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-repair-6`

Source report: `f4b34ccbdcef9e14d6bfdfb17f4e28ff2dec4e94`

Repaired candidate: `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d`

Live URL: https://clinic-reminder-proof.sociobot.in

## Result

All verification-7 findings are repaired. The artifact remains a Vite/Svelte
web client served by the Rust/axum backend in one container. The deployment
keeps one replica and the existing Azure Files mounts at `/durable` and
`/backups`.

## Reproduction before repair

The exact verifier boundary was reproduced against the unmodified report
commit after `npm ci`:

- valid JSON with `Content-Type: text/plain` on the demo assignment write
  returned plain-text `415` with `Expected request with Content-Type:
  application/json`;
- a valid 17 KB JSON body on `/api/v1/billing/checkout` returned `401` instead
  of `413`;
- that `401` body said `available-in-response-header`, but there was no
  `X-Request-Id` header.

## Repairs

- All demo and managed API request bodies now pass through a 16 KB transport
  limit before authentication. JSON extractor `400`, `415`, `422`, and `413`
  failures use one JSON problem shape.
- Every generated problem, including `401` and rate-limit responses, receives
  a unique UUID. The same value is returned in the body and `X-Request-Id` and
  is written to the structured service log.
- Dispatch accepts one `reminder_id` and rejects client-supplied campaign copy.
  The fixed reminder content contains only the patient first name, clinic,
  appointment time, and opt-out instruction.
- Signed fixture identities now prove owner-scoped export, cross-tenant denial,
  organization-confirmed deletion, and post-delete absence. The public export
  representation also omits the stored encrypted billing entitlement.
- `.factory/claims.json` now contains executable claims for no marketing
  campaigns, signed-in export/delete, and explicit theme selection. The
  request-protection claim and sandbox now cover wrong content type, a
  protected oversized write, and correlated request IDs.
- The header now offers System, Clinic daylight, and After hours. Explicit
  choices persist locally, update `theme-color`, and retain the tested light
  and dark token sets.
- The static duplicate description was removed. Svelte owns the single
  route-rendered description meta element.

## Verification evidence

- Clean install: `npm ci` — 87 packages, 0 vulnerabilities.
- Claims gate: every exact command in `.factory/claims.json` passed
  independently (24/24).
- `npm test` — 6 Vitest, 27 Rust, and 31 Chromium tests passed.
- `npm run check` — Svelte 0 errors/warnings, rustfmt clean, clippy with
  warnings denied.
- `npm run build` — `dist/` and the optimized API binary produced. Initial
  entry JS is 81.35 KB raw / 28.34 KB gzip; CSS is 25.30 KB raw / 5.44 KB
  gzip. The 271.99 KB MSAL chunk remains lazy.
- Playwright covered desktop and 390×844 mobile, 200% text, keyboard focus,
  reduced motion, route/back behavior, offline demo reads, privacy request
  logging, local links, console/request failures, and styled HTTP 404.
- Axe integration scanned seven public routes in both light and dark themes:
  zero serious or critical findings.
- `/opt/fleet/lib/verify-url.sh` on the production build found one `h1`, one
  main landmark, `lang=en`, no missing alt text, no unlabeled button, and no
  console error. Evidence is in `.factory/qa-artifacts/repair-6/local/`.
- Local mobile Lighthouse: Performance 99, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.17 s, LCP 1.41 s, TBT 95 ms, CLS 0.
- Production was built from the final committed source in ACR, applied to the
  existing Container App without replacing its storage topology, and checked
  for exact `/health` identity, one replica, both mounts, normalized boundary
  responses, auth configuration, and browser behavior.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
```

Run one contract exactly as the verifier does, for example:

```sh
npm run test:e2e -- --grep @claim:request-protection
```

## Known gaps

No release-blocking product gap remains. Live provider delivery, purchase
submission, and destructive deletion of a real clinic were not performed
because the work order supplied no clinic account, patient consent, provider
credential, or purchase authorization. Their boundaries are covered with
signed local identities and fixture providers/billing.

---

# Independent verification 8 handoff — PASS

Date: 2026-08-29 UTC
Candidate: `b8ef87c632ac95f5ef2d41dec162cba7639eddd5`
Live URL: https://clinic-reminder-proof.sociobot.in

**PASS — release accepted.** The live health endpoint reports the exact
candidate SHA; locally built entry JS and CSS match the deployed files by
SHA-256. The verifier ran `npm ci`, every exact `.factory/claims.json` command,
`npm test` (6 Vitest, 27 Rust, 31 Playwright), `npm run check`, and
`npm run build` successfully. Docker was unavailable in the verifier image;
the optimized package build and live container were verified instead.

The cold first screen plainly identifies the job, independent clinics as the
audience, and the one-click “Try it with sample data” action. The live sample
flow, fallback/consent exception, assignment/resolve/undo/reset, invalid API
inputs, 390px/200% text, keyboard skip/focus, reduced motion, offline read
state, axe serious/critical scan, headers/caching, and same-origin-only demo
requests all passed.

Observed live rate allowance: demo workspace creation permits 5 requests per
first-hop client in an hour, then responds 429 with `Retry-After: 3599`.
Other server bursts also returned 429 with `Retry-After: 1`; health is exempt.
The Entra configuration is exclusively the required Sociobot tenant.

One low-severity note only: an intentionally exhausted demo-create allowance
produces Chrome's generic console 429 resource message while the UI correctly
shows its retry state. Fresh normal loads are console-clean. See
`.factory/verification-8.md` for exact commands, claim results, evidence, and
known limits.
