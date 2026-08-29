# Verification handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-17`
Candidate: `3a341b7d34f6e734791ec37596295cda193374ed`
URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL — release blocked

The candidate passed clean local installation, all 40 repository browser
tests, all 34 Rust tests, type/format/lint checks, and the production build.
All 31 tagged browser claim assertions passed and the three separately named
Rust claim assertions passed. The release is nevertheless blocked because the
final composite deployment claim fails: `npm run verify:deployment:current`
reports that the active topology does not set both replica bounds to one.

Azure shows candidate revision `sf-clinic-reminder-proof--0000051` at declared
100% traffic, unhealthy, with `maxReplicas: 3` and no `/durable` or
`/backups` Azure Files mounts. Its own startup log says it refused unsafe
storage because those mounts are missing. Public `/health` and the footer are
therefore served by healthy fallback `2ec97d29b07279e15efb5e82caf002ffe63765e1`,
not the requested candidate.

Defects:

- Critical QA17-01: candidate traffic revision has no durable/backup mounts
  and permits three replicas.
- Critical QA17-02: live runtime identity is `2ec97d2…`, not `3a341b7…`.

The fallback build otherwise passed cold-read/demo, same-origin privacy
request logging, factory `verify-url.sh`, live Axe serious/critical scans,
390 px keyboard/reduced-motion checks, response-header/cache checks, and
observed demo rate limiting (five creations per hour with a `429` and
`Retry-After`).

## Recheck after repair

Deploy the exact full candidate image through the topology-aware deploy
command, then require one healthy 100% traffic revision with
`minReplicas=maxReplicas=1` and both Azure Files mounts. Confirm public
`/health` and the footer report the full candidate SHA, then run:

```sh
npm ci
npm test
npm run check
npm run build
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

Full evidence and reproduction detail: `.factory/verification-17.md`.
