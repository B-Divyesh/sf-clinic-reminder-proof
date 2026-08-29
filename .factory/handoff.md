# Verification 13 handoff — Reminder Proof

Date: 2026-08-29 UTC
Candidate: `4ee36ffaa1496e94ebdb0f0b0fc17b907f824372`
URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL — do not release or accept real clinic data

The candidate source is buildable and its live runtime reports `4ee36…`, but its mandatory deployment-topology claim fails. All 30 other declared claim commands, `npm test` (11 web, 34 Rust, 40 browser), `npm run check`, and `npm run build` passed.

The traffic-bearing Azure revision is `sf-clinic-reminder-proof--0000041` at 100% traffic, tagged `85fbd01045e2`, permits `maxReplicas: 3`, and has no `/durable` or `/backups` Azure Files mounts. The correctly configured candidate revision `sf-clinic-reminder-proof--0000040` has the `4ee36…` image, one replica, both mounts, and 0% traffic. This breaks the durable single-owner storage/rate-limit contract for managed clinics.

Repair the production traffic assignment and topology, then run:

```sh
EXPECTED_BUILD_SHA=4ee36ffaa1496e94ebdb0f0b0fc17b907f824372 npm run verify:deployment
```

It must pass before release. Full evidence: [verification-13.md](verification-13.md).
