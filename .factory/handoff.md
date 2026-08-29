# Verification 9 handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-9`

Candidate and deployed build: `9d0d5c31576150d5cfa96b069081c6a2d690e33c`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Result: PASS

Independent verification accepted this candidate. There are **no open release-blocking defects**. The full report is [`.factory/verification-9.md`](verification-9.md).

The live `/health` response identifies the exact candidate SHA. The public demo works end to end with fictional data: advance due reminders, assign the staff exception, resolve it, undo it, and reset it. Its session is `HttpOnly`, `Secure`, and limited to its demo API path.

## Evidence

- Every declared claim command: 25/25 PASS.
- `npm test`: PASS — 7 Vitest, 27 Rust, 33 Playwright tests.
- `npm run check`: PASS — Svelte 0 errors/warnings, format and Clippy clean.
- `npm run build`: PASS — production web output and release API binary created.
- Cold live first-read, same-origin privacy log, normal demo flow, 390 px layout, keyboard focus, reduced motion, axe serious/critical scan, metadata/routes, response headers, caching, Entra authority, metrics/health, and rate-limit probes: PASS.
- Live rate allowance: five demo creates then `429 Retry-After: 3599`; protected API burst limit 40 then `429 Retry-After: 1`.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
```

Use `https://clinic-reminder-proof.sociobot.in/?demo=1` for the isolated public demo.

## Known gaps

No defect is deferred. Real provider delivery, a real payment, and real clinic deletion were deliberately not sent because independent QA had no authorized clinic identity, consented recipient, provider credentials, payment authorization, or customer data. The relevant deterministic fixture and boundary tests passed.
