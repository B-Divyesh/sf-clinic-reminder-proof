# Verification 12 handoff — Reminder Proof

Date: 2026-08-29 UTC
Candidate: `a95a64b6f1ccba175adc34dec75a23ad38bb8974`
URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL

**Do not release or accept real clinic data.** The local candidate is buildable and all local tests pass, but the candidate production revision is unhealthy and the public URL does not serve the candidate.

- `npm ci`, all 31 declared claim commands, `npm test` (9 web + 34 Rust + 40 browser), `npm run check`, and `npm run build` passed locally.
- Cold first-read and one-click sample demo passed. Desktop/390 px demo, keyboard focus, reduced motion, privacy request log, Axe serious/critical scan, headers, auth authority, invalid-input recovery, and older-live rate-limit checks passed.
- `EXPECTED_BUILD_SHA=a95... npm run verify:deployment` failed because the active Container Apps template has `maxReplicas: 3`, not the mandatory one.
- Azure candidate revision `sf-clinic-reminder-proof--0000039` has 100% configured traffic but is `Unhealthy` / `ActivationFailed`, has no `/durable` or `/backups` Azure Files mounts, and logs: `required durable storage mounts are missing: /durable, /backups; refusing unsafe production storage`.
- Public `/health` instead reports prior build `aae2ef63533d5daf18c175feabb03aa482152d9f`.

Repair the deployment topology (one replica, both mounts), deploy a healthy candidate revision, then prove `/health` reports its exact SHA and rerun `EXPECTED_BUILD_SHA=<sha> npm run verify:deployment`. Full evidence: [verification-12.md](verification-12.md).
