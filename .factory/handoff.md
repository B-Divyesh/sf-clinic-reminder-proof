# Reminder Proof handoff

Current milestone: M1 public proof sandbox implemented locally, pending independent review and polish.

See [handoff-m1.md](handoff-m1.md) for shipped functionality, all verification evidence, deployment limitation, and the M2 handoff.

Source is pushed at `02b6b5a`. Live deployment remains pending because the work order exposes no container target for this slug and the production hostname did not resolve from the worker.

Quick verification:

```sh
npm ci
npm test
npm run check
npm run build
```

Public local demo: `http://127.0.0.1:8080/?demo=1` after `npm run build:web && npm run dev:api`.
