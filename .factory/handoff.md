# Verification handoff — FAIL

Candidate `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d` at https://clinic-reminder-proof.sociobot.in **FAILS release verification**.

The live `/health` SHA and Azure image match the candidate. All 21 declared claim commands, `npm test`, `npm run check`, and `npm run build` pass locally; the live demo, privacy/network checks, rate-limit probe, accessibility, mobile reflow, and hosted checkout also pass.

Release remains blocked by the deployed Container App configuration: active revision `sf-clinic-reminder-proof--0000024` has `maxReplicas: 3`, no volumes, and no `/durable` or `/backups` mounts. Managed clinic data and recovery files are therefore ephemeral and replica-local, and process-local client limits will multiply if scaled. Do not accept real clinic data.

Required operator action: deploy the checked-in single-replica configuration with the two Azure Files mounts, then prove authenticated persistence across a replacement and backup restoration. See `.factory/verification-6.md` for the exact commands, observed 5-per-hour demo allowance, and full evidence.
