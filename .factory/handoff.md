# Verification 15 handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-15`

Requested candidate: `ae20862641e90b0a265fc75ab76e5273159e7bef`

Reachable/tested source: `ae2086a2e672b9c61f3ea2867d16a2158ec8d94a`

Live URL: <https://clinic-reminder-proof.sociobot.in>

Full report: [verification-15.md](verification-15.md)

## Status: FAIL

Do not release and do not accept real clinic data. The exact candidate is absent from the clone and remote. Production does not match it: public health/footer serve older build `e831167…`, while Azure declares an unhealthy short-tagged `ae2086a2e672` revision at 100% traffic. That revision permits three replicas and has no durable or backup mounts. The mandatory claims gate is 30/31; `single-replica-durable-topology` fails.

## Evidence summary

- First-read and one-click demo: PASS on the older serving build.
- `npm ci`: PASS, 87 packages, 0 vulnerabilities.
- All literal claims commands: FAIL overall, 30 passed and 1 failed.
- `npm test`: PASS, 14 Vitest + 34 Rust + 40 Chromium.
- `npm run check`: PASS, zero Svelte diagnostics; rustfmt and Clippy clean.
- `npm run build`: PASS, produced `dist/` and release API.
- Live full browser suite: PASS, 40/40.
- Standalone axe: zero violations on four routes; light/dark Playwright Axe: zero violations.
- Lighthouse mobile: 98 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.498 s.
- Privacy: all 16 requests in the manual core flow were same-origin; no tracking/runtime third party.
- Rate limits: demo requests 1–5 returned 200; request 6 returned 429 with `Retry-After: 3599`. Protected API burst also returned 429 with `Retry-After`.
- Load smoke: 100/100 concurrent health responses returned 200 in 490 ms.
- Auth: exact Sociobot CIAM authority/client/tenant and PKCE redirect confirmed.
- Headers/cache: CSP, HSTS, nosniff, referrer/permissions/COOP present; hashed assets immutable for one year.
- Mobile/keyboard/reduced motion/200% text/404/offline-read/error recovery: PASS on the serving build.

## Blocking defects

| Severity | ID | Summary |
| --- | --- | --- |
| Critical | QA15-01 | Requested `ae208626…` commit is not present in the clone or GitHub remote. |
| Critical | QA15-02 | Live health/footer identify `e831167…`; candidate is not deployed. |
| Critical | QA15-03 | Azure revision `0000047` has `maxReplicas: 3`, no durable/backup mounts, is unhealthy, and is declared at 100% traffic. |
| High | QA15-04 | The deployment claim command is pinned to old build `e831167…`, not the candidate under test. |

## Recheck

Push the exact candidate, build/tag it with its full SHA, and deploy it only through the checked-in single-replica topology with both shares mounted. Confirm the candidate owns healthy 100% traffic and health/footer report the same full SHA. Then run:

```sh
npm ci
npm test
npm run check
npm run build
EXPECTED_BUILD_SHA=<exact-candidate> npm run verify:deployment
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

No product source code was modified during verification.
