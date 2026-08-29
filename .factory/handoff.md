# Verification handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-19`

Candidate: `9791736e1428961621a50ef8e9e1785c365e76b4`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL

Do not release or accept real clinic data. The mandatory
`single-replica-durable-topology` claim fails against Azure. The selected
100%-traffic revision `sf-clinic-reminder-proof--0000056` is unhealthy, permits
three replicas, uses a short image tag, and has no durable or backup mounts.
Its startup log says it refused unsafe production storage. The public site is
currently available only through healthy fallback revision `0000055`.

Full evidence and every claim result are in
[verification-19.md](verification-19.md).

## What passed

- Cold first-read and one-click sample demo.
- 30 of 31 exact manifest claim commands.
- `npm test`: 19 Vitest, 34 Rust, and 40 Playwright tests.
- `npm run check` and `npm run build`.
- Full live browser suite: 40/40 tests.
- Desktop, 390 px mobile, 200% text, keyboard/skip link, reduced motion,
  dark/light Axe scans, route/link/404 checks, and console checks.
- Same-origin demo privacy, security headers, immutable asset caching, input
  boundaries, API rate limits, health/build identity, and CIAM redirect.
- Mobile Lighthouse: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.5 s and CLS 0.001.
- Live initial assets remain within budget, and the full-SHA local JS is
  byte-identical to the live JS.

## Release blocker

`npm run verify:deployment:current` fails with:

```text
Error: deployment topology must set minReplicas and maxReplicas to 1
```

Fresh Azure state:

- `0000055`: healthy, full candidate tag, one replica, both Azure Files mounts,
  declared 0% traffic;
- `0000056`: unhealthy / ActivationFailed, short tag `9791736e1428`,
  `minReplicas: 1`, `maxReplicas: 3`, no volumes or mounts, declared 100% traffic.

The failed revision log says:

```text
required durable storage mounts are missing: /durable, /backups; refusing unsafe production storage
```

## Required next action

Roll out the exact full-SHA image using the checked-in deployment command so
the selected revision has one replica and both storage mounts. Do not use an
image-only update. Then require both commands to pass:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Operator check after repair

Confirm `https://clinic-reminder-proof.sociobot.in/auth/callback` remains
registered on the shared Sociobot Entra SPA. This verification did not use a
real clinic identity, provider credential, or paid subscription.
