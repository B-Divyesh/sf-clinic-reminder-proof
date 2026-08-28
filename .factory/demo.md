# Demo sandbox contract

Status: implemented in M1.

## Entry and isolation

- Public entry: `https://clinic-reminder-proof.sociobot.in/?demo=1`
- Stable route: `https://clinic-reminder-proof.sociobot.in/demo`
- No account, card, connector, or provider credential is required.
- `POST /api/v1/demo/workspaces` creates a random workspace with a 24-hour TTL in an HttpOnly, Secure, SameSite=Lax cookie scoped to the demo API.
- The compact cookie carries only a random ID and sample-state codes. It contains no clinic or patient data. Replica changes and process restarts cannot lose it.
- No demo operation can dispatch a message, call a provider, begin checkout, or read authenticated tenant state.

## Seed data

M1 must use fictional people and clearly label all delivery events “Simulated.” The canonical seed contains one clinic, two staff members, and five appointments:

| Patient alias | Appointment | Scenario | Expected outcome |
| --- | --- | --- | --- |
| Mina P. | Hygiene visit, 09:00 | SMS provider reports delivered | Delivery evidence |
| Jordan L. | Follow-up visit, 10:30 | Approved WhatsApp template is rejected; consented email succeeds | Ordered fallback evidence |
| Sofia R. | New patient visit, 14:00 | SMS opted out; no other channel allowed | Unassigned exception |
| Eli K. | Review visit, 15:30 | Email delivered; patient replies “YES” | Delivery and response evidence |
| Noor A. | Cleaning, next day 08:30 | Source cancels before reminder is due | Cancelled, no dispatch |

These labels are operational examples, not clinical records. The seed contains no diagnosis, treatment detail, date of birth, postal address, insurance information, real phone number, or real email address.

## Supported demo actions

- Inspect the delivery ledger and each reminder timeline.
- Advance deterministic simulated attempts.
- Observe a consent block and an ordered fallback.
- Assign the Sofia R. exception to Sam Rivera.
- Resolve it with “Called patient” and undo while safe.
- Reset the demo from the persistent banner.
- Choose “Start for real” to leave the isolated sample and open Sociobot Entra sign-in. The signed-in clinic workspace never reads the demo cookie.

## Reset

“Reset demo” deletes or expires the current workspace, requests a new random workspace, reloads the canonical seed, clears demo filter state, and restores focus to the demo `<h1>`. It does not mutate a shared seed or reuse an authenticated organization.

## Storage namespaces

- Server: no demo workspace data is retained. Each request reconstructs the canonical fictional sample from compact cookie state.
- Browser preferences: session keys are prefixed `demo:clinic-reminder-proof:<workspace-id>:` only. The cookie is HttpOnly and is not readable by page code.
- Authenticated clinic data uses the encrypted durable server store and is never queried while the demo banner is shown.

## Verification

Every M1 claim begins in a fresh browser context at `/?demo=1`. Tests record browser requests for the full flow and allow only the product origin. Provider and billing adapters are replaced by compile-time-disabled demo implementations, not network mocks in the browser. The seed and clock are deterministic in test mode. Test traces and screenshots go under `test-results/` and are never committed.

The reference command is `npm run test:e2e`; each claim tag can be selected with `npm run test:e2e -- --grep @claim:<id>`.
