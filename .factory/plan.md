# Reminder Proof venture plan

Status: **M1 complete — review 1 findings repaired and verified in polish 1**

Product slug: `clinic-reminder-proof`

Artifact: `web-with-backend`

Production URL: `https://clinic-reminder-proof.sociobot.in`

Last updated: 2026-08-28

This document is the delivery contract for Reminder Proof. A milestone builder must read this plan, `.factory/design.md`, every earlier handoff, and the latest review notes before changing product code. A milestone is complete only after its claims pass in a fresh demo sandbox and the milestone has passed review and polish.

## 1. Product requirements

### Customer and situation

The primary buyer is the owner or practice manager of an independent dental or outpatient clinic with one to ten locations. The daily user is a front desk coordinator who already works from an EMR or calendar. The clinic does not want another scheduling system.

Today, staff trust a reminder runner until a patient says no message arrived. They then search an EMR, a calendar, and one or more messaging dashboards. If WhatsApp approval fails, a sender account is suspended, a calendar is stale, or a reminder job crashes, staff fall back to calls and manual texts. Consent and opt-out checks are often implicit. There is no single record of what was attempted, what the provider returned, what the patient answered, or who owns the failure.

### Promise

**Every due reminder ends with delivery proof or a named staff owner.**

“Proof” means an append-only timeline of the scheduled reminder, consent decision, channel attempts, provider acknowledgements, patient responses, and any staff resolution. It does not mean guaranteed patient attention.

### The three jobs the product must nail

1. **Prove each outcome.** Import upcoming appointments from an existing calendar or EMR, schedule the approved reminder policy, and show a timestamped outcome for every due reminder.
2. **Use a safe fallback.** When one channel fails, try the next channel that the patient has consented to, while honoring opt-outs, quiet hours, and template approval.
3. **Give failures an owner.** Put blocked, exhausted, stale, and contradictory reminders into a front desk queue where staff can assign, act, add a non-clinical note, and resolve them.

### Product principles

- Be a reliability layer, not a replacement calendar.
- Never claim that queued means sent or that sent means seen.
- Prefer a visible exception over a silent retry loop.
- Store the least patient information needed to match an appointment and deliver a reminder.
- Treat consent, opt-out, and provider events as immutable evidence.
- Use plain operational language: appointment, reminder, attempt, delivered, failed, exception, owner.
- The demo is honest: provider behavior is simulated and every simulated event is labeled.

### Monetisation

All paid plans are monthly Dodo-backed subscriptions sold only through the Sociobot billing API. Messaging fees are passed through at the published channel cost; Reminder Proof adds no undisclosed margin. The demo requires no card.

| Tier | Price | Included | Intended customer |
| --- | ---: | --- | --- |
| **Clinic** | **$79 per location/month** | 1 location, 3 staff seats, 1 calendar/EMR connector, 12-month proof ledger, all approved channels | Independent clinic proving the wedge |
| **Practice** | **$199/month** | Up to 3 locations, 10 staff seats, 3 connectors, shared exception queue, 24-month ledger | Small multi-location practice |
| **Network** | **$499/month** | Up to 10 locations, 30 staff seats, 10 connectors, location policies, 36-month ledger, priority support | Regional outpatient group |

Every tier includes consent enforcement, accessibility, exports, delivery proof, and safety controls. Those are not upsells. Plan limits are enforced server-side. The product links to Sociobot-hosted checkout; it never embeds or calls Dodo directly. The final tier identifiers and recurring checkout contract must be recorded from the Sociobot product registration in M2 rather than guessed in client code.

### Success measures

Primary outcome, measured per clinic and over a rolling 30-day window:

- **≥99% accountable reminder rate:** `due reminders with (delivered evidence OR open/assigned/resolved exception) / all due reminders`. A cancelled appointment before the reminder becomes due is excluded and remains auditable.
- **15% relative reduction in reminder-related no-shows within 90 days:** compare the first four complete weeks after onboarding with a clinic-provided or imported four-week baseline. Display the sample size and never imply causation when the clinic does not supply outcome data.

Guardrails:

- 100% of opted-out channel attempts are blocked before provider dispatch.
- No cross-tenant reads in automated isolation tests.
- Provider webhook events are idempotent; duplicate events do not create duplicate attempts or responses.
- A due reminder becomes visible in the exception queue within five minutes after the policy is exhausted.

### Deliberately out of scope

- Clinical notes, diagnoses, treatment plans, prescribing, medical advice, or patient triage.
- A replacement EMR, patient record, calendar, booking page, or resource scheduler.
- Contact discovery, bought lists, marketing campaigns, or unapproved bulk messaging.
- Writing reminder text with a runtime model. Approved deterministic templates are safer and easier to audit. No runtime AI is planned; any future use must go through the Sociobot gateway and pass a separate privacy and claims review.
- Voice calls, inbound call recording, insurance verification, or payment collection from patients.
- Claims of HIPAA or UK GDPR “certification.” The product will provide technical controls and BAA/DPA readiness; clinic operators remain responsible for lawful configuration and consent.

## 2. Evidence and wedge

### Demand signals

| Date | Signal | What it establishes | Limitation |
| --- | --- | --- | --- |
| 2025-11-14 | [HN discussion: dental reminder builder and WhatsApp account loss](https://hn.algolia.com/api/v1/items/45927422) | A clinic-facing builder spent weeks on verification, was rejected, then lost an account after three template messages without a useful appeal path. One-channel dependence is an operational risk. | One builder’s account; it does not quantify the whole market. |
| 2026-05-29 | [OpenEMR issue #12307](https://github.com/openemr/openemr/issues/12307) | Reminder runners can fail around empty runs, recurring events, and status updates. “The job ran” is not enough; each reminder needs observable state. | An issue report is not prevalence data. |
| 2025-08-27 | [Cal.com issue #23391](https://github.com/calcom/cal.diy/issues/23391) | A displayed slot can disagree with actual assignee availability. Calendar synchronization failures surface late and cost staff time. | Scheduling availability is adjacent to reminders, not the same workflow. |

The three signals are independent and recent. Together they show recurring failure at the provider account, reminder runner, and source-calendar layers. Each failure can create a missed reminder, a no-show, or a manual callback.

### Incumbents and gap

OpenEMR and Cal.com own the source schedule. NexHealth and reminder vendors own messaging or patient engagement. WhatsApp, SMS, and email providers expose their own logs. None is neutral evidence across source and channel boundaries for a small clinic.

Reminder Proof sits beside the existing stack. It switches the buying question from “Which scheduler should we replace?” to “Can the front desk prove what happened before tomorrow’s list starts?” A clinic can begin with one read-only connector, existing approved templates, and its current providers. This lowers switching risk and makes value measurable through exception coverage and no-show outcomes.

### First validation before paid launch

Recruit five independent clinics across at least two jurisdictions. For two weeks, shadow 100 upcoming appointments per clinic without sending messages: import the schedule, reconcile existing provider events, and count silent failures and staff lookup time. Proceed with live paid delivery only if at least three clinics find ≥2 actionable exceptions per 100 appointments or save ≥30 front desk minutes per week, and at least two agree to pay the Clinic price after a supervised live trial.

## 3. Architecture

### Stack decision

- **Web:** Svelte 5, Vite, strict TypeScript, platform CSS, and small headless utilities only when native controls are insufficient. The ledger and exception queue need reactive state, but not React’s ecosystem weight.
- **API:** Rust 2021, axum, tokio, serde, tracing, tower-governor, and sqlx.
- **Database:** PostgreSQL in hosted environments because organizations, locations, workers, and webhook processing share state. Tests use an isolated PostgreSQL database; SQLite is not used as a production substitute.
- **Jobs:** a Tokio worker in the same deployable binary for the first release, using a PostgreSQL job table with `FOR UPDATE SKIP LOCKED`. Split the worker only after measured contention.
- **Deployment:** one container on Azure Container Apps. Axum serves the hashed Vite build and `/api/*` from the same origin. A second worker replica can run with `PROCESS_ROLE=worker`; the default `PROCESS_ROLE=all` remains functional with only `PORT` set.
- **Tests:** Vitest for pure UI/domain logic, Rust unit and integration tests for service boundaries, and Playwright 1.58.2 for every claim and browser journey.

The root `npm` scripts are the product’s portable build contract. `npm test` runs frontend and Rust tests. `npm run build` produces the web app in `dist/` and a release API binary. No runtime CDN script or font is allowed.

### Request and delivery flow

1. A connector poll or manual CSV import upserts an appointment using the tenant-scoped source ID and source version.
2. The scheduler evaluates the active, versioned reminder policy and records a reminder plus its due attempts.
3. Before each dispatch, the worker rechecks appointment status, consent, opt-out, quiet hours, and template approval.
4. A provider adapter sends with a deterministic idempotency key and stores the provider acknowledgement.
5. Signed provider webhooks append normalized events. Out-of-order events are retained and folded into the displayed state by deterministic precedence rules.
6. A terminal delivery closes the reminder. A retryable failure schedules the next allowed channel. An exhausted or unsafe reminder opens an exception and assigns it according to location rules.
7. Patient replies update the timeline and can open an exception; they never trigger medical advice or an automated clinical response.

### Repository shape

```text
apps/web/                       Svelte SPA and route-level UI
packages/design-system/         shared CSS tokens and component contract
services/api/                   axum API, static serving, worker entry point
services/api/src/routes/        HTTP boundary by domain
services/api/src/domain/        consent, policy, reminder state machines
services/api/src/db/            sqlx repositories and tenant scope helpers
services/api/src/providers/     SMS, email, WhatsApp adapters
services/api/src/jobs/          scheduler, dispatch, reconciliation, cleanup
services/api/migrations/        reversible PostgreSQL migrations
tests/e2e/                      Playwright journeys and claim tests
.factory/                       product, design, claims, demo, and handoffs
dist/                           reproducible Vite output; never hand-edited
```

### HTTP surface

Public routes stay usable without an account: `/`, `/demo`, `/?demo=1`, `/privacy`, `/terms`, and `/404`. Authenticated application routes live below `/app`. JSON endpoints live below `/api/v1`. `/health` returns the build SHA and is the only rate-limit exemption. `/metrics` requires an operator token or ingress restriction and must not include patient labels.

All writes accept an idempotency key, use JSON problem details for errors, and return a request ID. List endpoints use stable cursor pagination. Browser mutations require a validated Entra bearer token and same-origin checks. Provider webhooks require signature verification, a timestamp tolerance, and event-level idempotency.

### Data model and ownership

All mutable records use UUIDv7 IDs, `created_at`, `updated_at`, and an optimistic version where concurrent staff edits matter. Patient-facing times retain both UTC and the IANA timezone used to render them.

| Entity | Important fields | Ownership and retention |
| --- | --- | --- |
| `users` | Entra `oid`, display name, last sign-in | Global identity keyed only by stable `oid`; email is contact metadata, never the key. |
| `organizations` | legal/display name, jurisdiction, retention policy | Top-level tenant. No domain object is queryable without `organization_id`. |
| `memberships` | organization, user, role (`owner`, `manager`, `staff`, `viewer`), state | Unique per user and organization; role checks are server-side. |
| `locations` | organization, name, timezone, quiet hours, escalation rule | Billing and policy boundary. |
| `subscriptions` | organization, Sociobot entitlement reference/hash, tier, status, checked time | No card or Dodo secrets. Reverified at most daily and on protected plan changes. |
| `connectors` | location, kind, encrypted credential reference, cursor, health | Credentials encrypted with a generated-at-first-boot key persisted under `/data`; never returned to the browser. |
| `patient_refs` | organization, source-scoped opaque ID, preferred locale, encrypted destination fields | No diagnosis, chart, address, date of birth, or clinical note. Delete or pseudonymize per retention policy. |
| `channel_consents` | patient ref, channel, status, source, captured time, evidence reference | Append-only changes; effective state is derived. Opt-out wins over consent. |
| `appointments` | location, source ID/version, patient ref, start/end, status, non-clinical service label | Upserted idempotently; source cancellation prevents future dispatch. |
| `reminder_policies` | location, version, offsets, ordered channels, quiet-hour behavior | Published versions are immutable; reminders point to the version evaluated. |
| `templates` | location, channel, locale, provider approval ID, version, body variables | No free-form medical content. Published versions are immutable. |
| `reminders` | appointment, policy version, due time, folded status | One record per appointment/policy occurrence with a deterministic key. |
| `attempts` | reminder, channel, template version, consent snapshot, state, idempotency key | Append-only evidence apart from a deterministic folded state. |
| `provider_events` | attempt, provider event ID, received/occurred time, normalized/raw code | Append-only; raw payload is redacted and short-lived where possible. |
| `patient_responses` | reminder, channel, normalized response, encrypted body, received time | Reply body has the shortest configurable retention; responses can open exceptions. |
| `exceptions` | reminder, reason, severity, owner membership, state, resolution code, due time | Staff task. Resolution notes forbid clinical content and are audited. |
| `audit_events` | organization, actor, action, target, before/after summary, request ID | Append-only, tamper-evident hash chain per tenant; never contains destinations or message bodies. |
| `notification_preferences` | membership, digest and escalation choices | Staff operational email only; opt-in except required account/security mail. |
| `export_jobs` | organization, requester, state, expiry, object reference | Generated on request, encrypted, signed short-lived download, deleted after 24 hours. |

Tenant isolation is enforced in three layers: an authenticated request context, repository methods that require an `OrganizationId`, and PostgreSQL row-level security using a transaction-local tenant setting. Integration tests create two tenants and attempt reads and writes across every tenant-owned repository.

### Authentication and authorization

Use the shared Sociobot Microsoft Entra External ID tenant through `@azure/msal-browser`, authorization code + PKCE, `loginRedirect`, and `acquireTokenSilent`. Cache only in `sessionStorage`. Public and demo routes never require sign-in.

Defaults, overridable by the exact environment variables shown:

- `ENTRA_TENANT_ID=35c6fe40-0ec0-46b6-98c6-213ad4de6650`
- `ENTRA_TENANT_SUBDOMAIN=sociobotcustomers`
- `ENTRA_CLIENT_ID=25c704f4-465a-47af-80ab-2c489466b697`
- authority `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`
- redirect `https://clinic-reminder-proof.sociobot.in/auth/callback`
- scopes `openid profile email`

The backend loads OIDC discovery at startup, caches its JWKS for one hour, and validates RS256, discovery issuer, audience, tenant, `exp`, and `nbf`. It keys the user by `oid` and returns `401` with `WWW-Authenticate: Bearer` for invalid tokens. Roles are local tenant memberships, not claims trusted from the client.

### Billing

M2 registers `clinic`, `practice`, and `network` monthly products with the Sociobot billing catalog and uses its returned recurring checkout identifiers. The browser starts checkout through a same-origin `/api/v1/billing/checkout?tier=…` endpoint, which allowlists the tier and redirects only to `https://api.sociobot.in/api/v1/...`. Dodo remains behind Sociobot and is never linked directly.

On return, the backend exchanges or verifies the Sociobot entitlement, stores only its stable reference or token hash, attaches it to the organization, and strips sensitive query values from browser history. Entitlements are rechecked at most daily, reconciled by signed webhook where supported, and fail closed for new paid operations while preserving read/export access. A cancelled or failed subscription never blocks accessibility, export, opt-out, or safety behavior. Billing tests use the Sociobot pilot endpoint and recorded contracts; they never make a live charge in CI.

Messaging usage is recorded from accepted provider attempts, reconciled against provider receipts, and reported to the Sociobot billing adapter once with an idempotency key. Until usage billing is confirmed in the factory contract, the UI labels fees “provider messaging charges” and does not invent a price.

### AI decision

No runtime AI feature is justified for M1–M6. The important decisions—consent, timing, channel fallback, delivery state, and exception ownership—must be deterministic and auditable. Clinics choose approved templates rather than generating patient messages. If later research shows value in summarizing a large exception queue, it must be explicit, optional, reversible, exclude message bodies by default, use only `https://api.sociobot.in/v1`, rate-limit spend, and add fixture-backed claims before launch.

### Background jobs and failure handling

The database job table stores kind, tenant, due time, attempt count, lease owner/expiry, idempotency key, and last safe error. Jobs are leased with `SKIP LOCKED`, heartbeated, retried with capped exponential backoff plus jitter, and moved to a dead-letter state after policy exhaustion. The scheduler uses a uniqueness constraint, so restarts cannot create duplicate reminders. Dispatch idempotency keys are stable across retries. A reconciliation job polls providers when webhooks are late. Cleanup deletes expired demo tenants, raw webhook payloads, and exports.

Every job transition emits a structured log and counter. A job failure that affects a reminder also opens or updates a visible exception; operations must not depend on log access to discover patient-impacting failures.

### Files, email, and notifications

CSV imports are streamed, validated, and discarded after normalized rows are committed; rejected rows are returned as a downloadable report. Production exports use private Azure Blob-compatible object storage only when configured; the default single-container setup can stream small exports directly and still starts with only `PORT`. Connector and encryption secrets are generated with a CSPRNG at first boot and persisted under `/data`, with optional environment overrides.

Email to staff is transactional: security notices, connector failures, assigned exception alerts, and requested digests. It is opt-in except required account/security notices. Patient reminder email is a delivery provider channel governed by the same consent and template rules as SMS or WhatsApp; it is not marketing mail.

### Observability and operation

- `/health` returns status and `BUILD_SHA`; readiness also checks database connectivity after persistence is introduced.
- `/metrics` exposes low-cardinality Prometheus counters and histograms: reminder outcomes, exception age, provider latency, webhook failures, job lag, and HTTP status. Never label by patient, message, appointment, or raw tenant ID.
- JSON logs include timestamp, level, request ID, route template, tenant pseudonym, actor pseudonym, and safe error code. They exclude destinations, template bodies, access tokens, and raw provider payloads.
- OpenTelemetry trace propagation is accepted at boundaries. Sampling is configurable and defaults to errors plus a small request sample.
- Service target after M3: 99.9% API availability monthly; 99% of due reminders reach delivery proof or an exception within five minutes of policy exhaustion.
- Alerts: scheduler lag >2 minutes, webhook verification failures, dead-letter growth, database saturation, or accountable reminder rate below 99% for 15 minutes.

### Rate limits

Rate limiting is mandatory on every server endpoint except `/health`, keyed by the first valid `X-Forwarded-For` hop behind trusted factory ingress and otherwise the socket IP. A rejected request returns `429` and `Retry-After`.

- Public reads: 20 requests/second, burst 40 per IP.
- Demo workspace creation: 5/hour per IP; demo writes: 30/minute per workspace and 60/minute per IP.
- Authenticated reads: 30/second, burst 60 per user and tenant.
- Authenticated writes: 10/minute per user for settings/connectors; 60/minute for routine exception actions.
- Auth and billing starts: 5/10 minutes per IP and user.
- Imports: 3 concurrent per tenant, 20/day per location, size capped at 5 MB.
- Provider webhooks: 100/second per provider source plus signature verification and body-size limits.

### Privacy, security, backups, and export

- Encrypt transport, database volumes, provider credentials, patient destinations, and response bodies. Rotate data-encryption keys with versioned envelopes.
- Default retention: proof metadata 12 months on Clinic, according to tier for paid extensions; reply bodies 30 days; raw provider payloads 7 days; demo tenants 24 hours; exports 24 hours. Clinics may shorten retention.
- Record consent evidence and opt-outs longer where law requires, but pseudonymize destinations when no longer operationally needed.
- Nightly encrypted PostgreSQL backups with 30-day retention, weekly restore test in non-production, documented RPO ≤24 hours and RTO ≤4 hours for launch.
- Organization export includes locations, appointments with minimized patient references, consent history, reminder/attempt timelines, exceptions, and audit events in CSV/JSON. Account deletion is a confirmed, audited, delayed job with a seven-day recovery window unless legal retention applies.
- Security headers: restrictive CSP, HSTS at ingress, `X-Content-Type-Options`, `Referrer-Policy`, frame denial, and permission policy. No analytics beyond an aggregate, privacy-respecting page view with no patient or tenant attributes.
- Complete a threat model, subprocessor inventory, retention schedule, incident runbook, and BAA/DPA readiness checklist before live patient data. Do not market compliance as a certification.

## 4. Design system

The source of truth is `.factory/design.md`; executable tokens live in `packages/design-system/tokens.css`; component states live in `packages/design-system/component-inventory.json`.

### Direction: translucent pulse ledger

The interface looks like thin clinical trace sheets stacked over a deep slate desk. Each reminder becomes a horizontal evidence line. A small pulse travels once when a new attempt arrives, then settles into a precise mark. Translucency explains provenance and sequence; it is never a decorative blur. Warm amber interrupts the cool ledger when staff action is needed.

### Core tokens

| Role | Light | Dark | Use |
| --- | --- | --- | --- |
| Background | `#f3f7f5` | `#071519` | Quiet clinic paper / after-hours desk |
| Surface | `#fbfdfc` | `#0d2329` | Primary reading plane |
| Text | `#102a33` | `#e7f5f2` | Body and headings |
| Muted text | `#48636b` | `#aac3c5` | Secondary evidence |
| Accent | `#006a73` | `#63d8d3` | Current point, links, focus-adjacent action |
| Accent contrast | `#ffffff` | `#062025` | Text on accent |
| Success | `#216e4a` | `#6bddaa` | Delivered with text/icon label |
| Warning | `#8a4f00` | `#f4bb62` | Staff action needed |
| Danger | `#a12c3f` | `#ff8997` | Failed/blocked, never color alone |

Typography uses self-hosted Instrument Sans for UI and Fragment Mono for timestamps, IDs, and counts. The scale is 14, 16, 20, 25, 32, and 48 CSS pixels with body text never below 16 pixels. Spacing follows 4/8 pixels, radii are 6/12/20 pixels, and shadows are tinted slate rather than black.

Motion lasts 160–240 ms. Only a newly received attempt gets one 600 ms pulse from its originating marker. It never loops. Reduced-motion replaces travel with a border and opacity change.

### Component inventory (20)

1. App shell
2. Site header
3. Demo banner
4. Route announcer
5. Primary button
6. Secondary button
7. Text link
8. Field
9. Status chip
10. Consent badge
11. Filter bar
12. Ledger timeline
13. Attempt row
14. Pulse marker
15. Exception card
16. Assignment control
17. Confirmation dialog
18. Inline notice / toast
19. Empty/loading/error state
20. Evidence drawer

Each component contract includes default, hover, focus-visible, disabled, busy, empty, error, and reduced-motion behavior where relevant. The inventory file assigns each component to its first milestone.

### Five key screens

1. **Landing and live preview:** an asymmetrical two-column first screen. Plain copy and the sample-data action occupy the left. The right is a real, clipped ledger preview, not a decorative dashboard image. On mobile, the action and three facts appear before the preview.
2. **Today’s delivery ledger:** a date rail and outcome summary lead into appointment rows. Expanding a row reveals the evidence timeline without losing place. Filters collapse into a labeled sheet on narrow screens.
3. **Reminder detail:** source appointment, consent snapshot, policy version, attempts, provider events, and patient response read top to bottom as one chain. The exception action sits beside the exact break in the chain.
4. **Exception queue:** staff scan reason, appointment time, remaining safe action, age, and owner. Assignment is inline. Resolving requires a specific resolution code and offers undo while safe.
5. **Connection and policy setup:** a step list shows source health, allowed channels, consent mapping, template approval, quiet hours, and a dry-run result before activation. Billing is a separate settings section and never interrupts safety setup.

### State, responsive, and accessibility rules

- Empty: say what will appear and provide the one next action. The ledger empty state distinguishes “nothing due” from “connector has not synced.”
- Loading: preserve row geometry with quiet skeletons; include a text status for assistive technology. Never replace the whole screen after the first load.
- Error: say what failed, whether reminders are affected, and what to do. Keep prior evidence visible when safe.
- Offline: keep the last clearly timestamped ledger read-only. Never queue a dispatch from an unconfirmed browser action.
- 390–767 px: one column, summary before filters, 44 px controls, evidence drawer becomes a full-screen dialog, dense metadata moves behind “Show details.”
- 768–1199 px: two regions; the ledger is primary and the detail drawer overlays without shifting rows.
- ≥1200 px: ledger and evidence drawer can remain side by side; maximum reading width is 1440 px.
- One `<h1>` per route, ordered headings, landmarks, a skip link, native controls first, visible 3:1 focus indicators, 4.5:1 text contrast, 44×44 targets, text zoom to 200%, and no information conveyed only by hue or motion.
- Route changes update the title, focus the new `<h1>`, announce it politely, and restore back/forward scroll. All titles follow `.factory/design.md`.

## 5. Milestones

Each milestone fits one focused builder session. If a dependency cannot be completed safely in that window, the builder ships the smaller honest vertical slice, records the gap, and does not expose an unfinished paid or delivery path.

### M1 — Public proof sandbox

**Status:** complete — review 1 findings repaired and verified in polish 1

**Outcome:** A visitor can understand the product and complete the core proof workflow with realistic sample appointments. No real message is sent and no account is required.

**Routes and screens**

- `/` — landing page in the required site order with a real ledger preview and exact pricing.
- `/?demo=1` — canonical one-click demo entry; it may normalize to `/demo` while preserving back behavior.
- `/demo` — seeded delivery ledger with persistent demo banner.
- `/demo/reminders/:id` — sample reminder evidence and staff exception actions.
- `/privacy`, `/terms`, `/404` — complete, styled, linked pages.
- `/health`, `/api/v1/demo/workspaces`, `/api/v1/demo/*` — health and isolated demo API.

**Scope**

- Implement the design system components required by these routes.
- Seed one fictional clinic, two staff members, five appointments, and at least these outcomes: first-channel delivery, provider rejection followed by consented fallback delivery, opt-out block with an exception, patient reply, and source cancellation.
- Provision a random, rate-limited, 24-hour demo workspace. The signed demo cookie can reach only that workspace. Label all provider events “Simulated.”
- Let a visitor advance a deterministic reminder, inspect its timeline, assign an exception, resolve it, undo a safe resolution, and reset the complete demo.
- Add all route metadata, original code-drawn ledger art, responsive states, privacy/terms content, sitemap, robots, security headers, and SPA fallback.
- Document the exact demo dataset and reset contract in `.factory/demo.md`.

**Claims**

The executable M1 contract is `.factory/claims.json`: `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline`, `exception-ownership`, `demo-reset`, `minimal-reminder-content`, and `public-price`.

**Tests**

- One Playwright test tagged exactly once for each `@claim:<id>`, starting from a fresh browser context and the public demo entry.
- Browser journeys for keyboard-only entry, mobile 390 px layout, back/forward route focus, deep reload, offline read-only state, invalid demo ID, expired demo, and API error recovery.
- Rust integration tests for workspace isolation, TTL cleanup, deterministic transitions, body limits, 429 + `Retry-After`, and health build SHA.
- Vitest tests for reminder fold precedence, consent decisions, state copy, and token/component contracts.
- Automated axe scan for every route; link crawl; console and failed-request assertion.

**Definition of done**

- All nine claims pass from `?demo=1`; the trace/screenshot path is recorded.
- `npm test`, `npm run check`, and `npm run build` pass from a clean clone; `dist/` is produced.
- No real provider call, tenant data, authentication, or billing state is reachable from demo mode.
- One `<h1>` and correct title per route; serious/critical axe findings are zero; keyboard and 390 px review pass.
- Initial JS ≤150 KB gzip, CSS ≤50 KB, self-hosted fonts ≤120 KB, LCP <2.5 s, INP <200 ms, CLS <0.1; Lighthouse mobile ≥90 performance and ≥95 accessibility.
- `.factory/copy-audit.md`, `.factory/demo.md`, and `.factory/handoff-m1.md` contain measured evidence. Landing copy describes only simulated capability.

### M2 — Accounts, durable clinic data, and subscriptions

**Status:** implemented in repair 2 — Entra account, encrypted durable workspace, onboarding, and Sociobot checkout/verification

**Outcome:** A clinic owner can sign in, create an isolated organization and location, complete safe onboarding, choose a monthly plan in Sociobot checkout, and return to durable account data.

**Routes and screens**

- `/sign-in`, `/auth/callback`
- `/onboarding/clinic`, `/onboarding/location`, `/onboarding/staff`
- `/app`, `/app/settings/members`, `/app/settings/billing`, `/app/settings/privacy`
- `/api/v1/me`, `/api/v1/organizations`, `/api/v1/locations`, `/api/v1/memberships`, `/api/v1/billing/*`, `/api/v1/exports`, `/api/v1/account-deletion`

**Scope**

- Add Entra CIAM PKCE sign-in with the exact tenant contract above. Confirm/register the production redirect URI and record operator action if it is pending.
- Add reversible PostgreSQL migrations for users, organizations, memberships, locations, subscriptions, audit events, notification preferences, and export jobs; enable RLS and tenant-required repositories.
- Add onboarding with jurisdiction, timezone, retention choice, staff roles, and an explicit statement that clinical notes must not be entered.
- Register the three recurring plans in the Sociobot pilot catalog, implement same-origin checkout and return, verify entitlements server-side, and handle active, grace, past-due, cancelled, revoked, and gateway-unavailable states.
- Add organization export and delayed deletion. Keep the public demo operational and isolated.

**Claims**

- `ciam-sign-in`: “Sign in with the shared Sociobot customer account.”
- `tenant-isolation`: “Clinic data stays inside its organization.”
- `durable-onboarding`: “Clinic and location settings remain after sign-out and sign-in.”
- `subscription-price`: “Clinic costs $79 per location each month.”
- `data-export`: “An owner can export the clinic’s stored data.”
- `account-deletion`: “An owner can schedule account deletion with a seven-day recovery window.”

**Tests**

- Recorded OIDC/JWKS contract tests plus a staging Entra smoke test; invalid issuer, audience, tenant, signature, expiry, and `nbf` return 401 with the correct header.
- Two-tenant API and repository tests for every tenant-owned table and role boundary.
- Pilot billing fixtures for checkout tier allowlisting, active and revoked entitlement, duplicate return, gateway timeout, and daily verification cache. No live charge in CI.
- Playwright onboarding, sign-out/in persistence, plan selection, export download, delete/cancel-delete, demo regression, keyboard, mobile, and axe tests.
- Migration up/down test and backup/restore smoke with seeded tenant data.

**Definition of done**

- A new user reaches the app with an organization and chosen plan in under five minutes on the pilot flow.
- No token, provider secret, raw license, or card data is stored in client logs or application tables.
- Tenant and role isolation tests pass; exports are tenant-complete and cross-tenant empty.
- Billing failure preserves read/export and safety actions; it cannot create duplicate subscriptions.
- CI, clean build, accessibility, performance, demo claims, migration, and handoff gates pass.

### M3 — Live intake, consented fallback, and staff ownership

**Status:** implemented in repair 2 — signed calendar intake, consent-aware Twilio/Resend dispatch, receipt-driven fallback, and shared exceptions

**Outcome:** A subscribed clinic can import or connect a basic read-only schedule, send consented email/SMS reminders with one ordered fallback, and work every failed outcome from one queue.

**Routes and screens**

- `/app/setup/source`, `/app/setup/consent`, `/app/setup/channels`, `/app/setup/policy`, `/app/setup/dry-run`
- `/app/ledger`, `/app/reminders/:id`, `/app/exceptions`, `/app/exceptions/:id`
- `/api/v1/connectors/*`, `/api/v1/imports/*`, `/api/v1/policies/*`, `/api/v1/templates/*`, `/api/v1/reminders/*`, `/api/v1/exceptions/*`, `/api/v1/webhooks/:provider`

**Scope**

- Ship validated CSV intake and one read-only ICS feed connector. Store source cursors, expose last successful sync, and surface stale or contradictory events.
- Map consent and opt-out evidence per channel. Run a dry preview that shows exactly which reminders would send, block, or become exceptions before activation.
- Add versioned templates and one policy shape with an offset, ordered email/SMS fallback, quiet hours, retry caps, and source cancellation checks. Defer policy variants and WhatsApp to M5.
- Add email and SMS provider adapters behind a typed interface, using clinic/factory-approved credentials stored only server-side. Each provider supports deterministic idempotency and signed webhook verification.
- Add the worker/outbox, reconciliation, proof ledger, patient response handling, assignment, resolution, undo where safe, and operational alerts. Record messaging usage idempotently for later pass-through billing.
- Never infer consent, never auto-reply with medical content, and never send from demo mode.

**Claims**

- `source-sync`: “Upcoming appointments arrive from the connected source with visible sync health.”
- `consent-preview`: “A dry run shows which channel is allowed before activation.”
- `live-consent-guard`: “An opted-out channel is blocked before provider dispatch.”
- `live-fallback`: “A retryable failure uses the next consented channel in the clinic’s policy.”
- `provider-proof`: “The ledger shows provider acknowledgements and delivery events without calling queued ‘delivered.’”
- `reply-capture`: “Patient replies appear on the matching reminder timeline.”
- `owned-exception`: “An exhausted reminder creates an exception that staff can assign and resolve.”

**Tests**

- Contract fixtures for ICS recurrence boundaries, CSV rejected rows, SMS/email requests, signed webhooks, duplicates, late/out-of-order events, timeouts, and account suspension.
- Domain property tests for consent precedence, quiet hours across DST, policy version immutability, state folding, retries, cancellation races, and idempotency.
- Worker integration test through Postgres: imported appointment → due reminder → failed first channel → successful fallback → complete evidence, plus exhausted path → owned exception.
- Playwright setup dry-run, ledger, reply, assignment/resolution, offline read-only, keyboard, mobile, and all earlier claim regressions.
- Rate-limit, payload-size, SSRF/connector URL, secret-redaction, and cross-tenant webhook tests.

**Definition of done**

- A supervised pilot clinic can connect/import, dry-run, activate, send through SMS and email, and prove the full chain.
- Provider outage, invalid credential, stale source, missing consent, opt-out, cancellation, duplicate webhook, and exhausted fallback are visible and safe.
- Every due reminder in seeded acceptance data has delivery evidence or an exception; no silent terminal state exists.
- Usage records reconcile one-to-one with accepted sends; no actual messages are sent in automated tests or demo.
- CI, security, accessibility, performance, demo regression, load smoke at 100 requests/second, and handoff gates pass.

### M4 — Operations, exports, and clinic control

**Status:** implemented core in repair 2 — metrics, export/delete, audit events, limits, and encrypted persistence; scheduled backups remain an operator concern

**Outcome:** Clinic managers and operators can run the service without database access, measure the promise honestly, and recover from common failures.

**Routes and screens**

- `/app/operations`, `/app/reports`, `/app/settings/notifications`, `/app/settings/retention`, `/app/audit`
- `/operator/tenants`, `/operator/providers`, `/operator/jobs` behind a separate operator authorization boundary
- `/api/v1/reports/*`, `/api/v1/audit/*`, `/api/v1/notifications/*`, `/api/v1/operator/*`, `/metrics`

**Scope**

- Add connector/provider health, scheduler lag, dead-letter visibility, safe job retry, and tenant-pseudonymous operator diagnostics. Operators cannot read message bodies or destinations.
- Add staff digests and immediate assigned-exception notices with per-user preferences and unsubscribe where applicable.
- Add CSV/JSON exports for ledger, exceptions, consent evidence, audit events, and outcome reports; preserve accessibility and safety regardless of plan.
- Add accountable reminder rate and clinic-entered/imported no-show baseline comparison with sample size, date range, and caveats.
- Add retention controls, pseudonymization/deletion jobs, backup restore drill tooling, audit search, and incident runbook.

**Claims**

- `accountable-rate`: “The report shows what share of due reminders have proof or an exception.”
- `exception-alert`: “Assigned staff can receive an exception email without patient message content.”
- `ledger-export`: “Managers can export the filtered ledger as CSV and JSON.”
- `audit-history`: “Owners can see who changed policies, assignments, and consent mappings.”
- `safe-operator-view`: “Operators can diagnose delivery systems without viewing patient destinations or message bodies.”

**Tests**

- Report formula fixtures including cancellation exclusions, timezones, empty samples, and late events.
- Notification fixtures for opt-in/out, deduplication, redaction, retry, and broken email transport.
- Export schema/content/authorization, retention, deletion, audit immutability, and operator redaction tests.
- Backup and point-in-time restore drill in CI or scheduled staging; documented RPO/RTO evidence.
- Playwright manager reports, settings, audit, operator views, keyboard/mobile/axe, and all previous claims.

**Definition of done**

- A clinic manager can explain the 99% numerator and denominator from exported rows.
- Operators can find and safely retry a failed job without production database access.
- Notification, export, retention, deletion, restore, and audit paths work in staging failure drills.
- Privacy review confirms no patient content in staff subject lines, metrics, operator views, or logs.
- CI, performance, accessibility, demo regression, and handoff gates pass.

### M5 — Approved WhatsApp and EMR integration

**Status:** implemented core in repair 2 — approved WhatsApp templates and a vendor-neutral signed EMR/calendar webhook; named EMR adapters remain future work

**Outcome:** A validated pilot clinic can add approved WhatsApp fallback and one high-demand EMR/calendar connector without weakening consent or proof semantics.

**Routes and screens**

- `/integrations`, `/app/integrations`, `/app/setup/channels/whatsapp`, `/app/setup/source/:kind`
- `/api/v1/integrations/*`, `/api/v1/connectors/*`, `/api/v1/webhooks/whatsapp`

**Scope**

- Add the next connector chosen from pilot demand (OpenEMR-specific FHIR profile or Cal.com API), using recorded contracts and a visible sync-health boundary.
- Add approved WhatsApp templates and provider adapter with account-health visibility, deterministic idempotency, signed webhook verification, and explicit suspension/rejection handling.
- Extend policy versions to support WhatsApp in an ordered fallback without migrating or changing already-scheduled reminder evidence.
- Add a connector/channel preflight that checks credentials, template approval, consent mapping, source read-only scope, and a no-send dry run.
- Publish the selected connector and WhatsApp setup guides. Keep all credentials server-side and all demo providers disabled.

**Claims**

- `second-connector`: “The selected pilot system syncs appointments with visible health and no write-back.”
- `approved-whatsapp`: “Approved WhatsApp templates can be used only when channel consent is recorded.”
- `whatsapp-fallback`: “A WhatsApp rejection or suspension moves to the next allowed channel or opens an exception.”
- `integration-preflight`: “Setup shows source, consent, credential, and template problems before activation.”

**Tests**

- Connector contract, cursor, replay, stale-data, and no-write assertions.
- WhatsApp request and webhook fixtures for approved/unapproved template, consent/opt-out, duplicate, late event, throttling, rejection, and account suspension.
- Policy-version regression proving old reminders retain their original channel order.
- Preflight Playwright flows for ready, missing consent mapping, invalid credential, unapproved template, stale source, keyboard, mobile, and axe.
- Full previous-claim, tenant-isolation, redaction, rate-limit, no-demo-provider-call, and 100 requests/second load regression.

**Definition of done**

- Pilot evidence selects the connector; no speculative integration is shipped.
- The selected connector has a complete read-only permission statement, replay fixtures, and visible last-good sync.
- WhatsApp cannot send with missing consent or an unapproved template; suspension never becomes a silent failure.
- The public demo and all previous workflows remain functional without a provider call.
- All claims, CI, performance, accessibility, security review, review/polish, and handoff gates pass.

### M6 — Installability and measured growth

**Status:** planned

**Outcome:** A clinic can invite the front desk, install the product for daily use, and share aggregate proof without exposing patient data.

**Routes and screens**

- `/app/install`, `/app/settings/invites`, `/app/reports/share/:token`
- `/api/v1/invites/*`, `/api/v1/report-links/*`

**Scope**

- Add an installable PWA shell for fast front desk access. Offline mode is read-only except local filter state; dispatch and resolution require a confirmed server response.
- Add role-scoped staff invitations through Entra and single-use, expiring invitation state.
- Add revocable, expiring report links containing aggregate outcomes only—never names, destinations, message bodies, or appointment-level rows.
- Add a consent-safe referral from the post-value moment and a privacy-respecting aggregate page count/conversion funnel with no patient or tenant identifiers.
- Publish the security/BAA/DPA readiness packet and migration checklist.

**Claims**

- `installable-shell`: “The app can be installed and the last ledger view remains clearly read-only offline.”
- `staff-invite`: “An owner can invite staff into a chosen location and role.”
- `safe-report-link`: “A manager can share a revocable aggregate report without patient-level data.”
- `demo-always-available`: “Anyone can still try the sample clinic without an account or provider call.”

**Tests**

- PWA installability, versioned cache, offline reload/read-only behavior, update recovery, and storage cleanup.
- Invitation expiry/replay/role/tenant tests and aggregate report link revocation/content scan.
- Privacy request audit allowing only same-origin plus explicitly approved Entra/Sociobot endpoints; no third-party trackers.
- Full browser regression on Chromium mobile/desktop plus smoke on WebKit and Firefox, all claims, link crawl, axe, Lighthouse, and 100 requests/second load smoke.

**Definition of done**

- Install, invite, and aggregate sharing are usable on a 390 px phone and have safe revocation paths.
- The public demo and all previous workflows remain functional through clean, expired, offline, and provider-failure states.
- The launch packet contains threat model, subprocessor list, retention schedule, incident contacts, backup evidence, provider setup, and honest compliance language.
- All claims, CI, performance, accessibility, security review, final review/polish, and handoff gates pass.

## 6. Risks and experiments

| Risk or unknown | Why it matters | Experiment that retires it | Decision gate |
| --- | --- | --- | --- |
| Clinics may not pay for proof separate from their EMR. | The wedge could be a feature, not a company. | Two-week shadow ledger with five clinics; measure exceptions and staff lookup time, then request a signed $79/month pilot intent. | Before M3 live sends; require the validation threshold in the evidence section. |
| The Sociobot billing API’s recurring contract may differ from the one-time license contract. | Invented parameters would break checkout or entitlement. | In M2, register three pilot products/tiers, record the returned IDs and webhook/verify fixtures, and run one refundable test subscription with operator approval. | Do not expose checkout until contract tests and return/revoke paths pass. |
| Entra redirect URI may not be registered. | Sign-in cannot complete in production. | Verify the exact callback against the shared SPA before M2 review. | Operator registers it; public demo remains available meanwhile. |
| WhatsApp verification or account suspension can block the intended fallback. | It is a source of the original pain. | Run approved-template sends in a clinic-owned test account and inject suspension/429 fixtures; prove SMS/email fallback and a visible exception. | Never make WhatsApp the only policy channel; label unavailable setup clearly. |
| Consent rules vary by jurisdiction and channel. | Unsafe dispatch creates legal and patient harm. | Review the consent model and default templates with counsel in the first two pilot jurisdictions; test opt-out precedence and evidence export. | Start live delivery only in reviewed jurisdictions; unknown jurisdiction defaults to blocked. |
| Calendar/EMR data is stale or contradictory. | A technically delivered reminder can still be wrong. | Shadow sync against 100 appointments per clinic; measure late cancellation/update rate and set a staleness threshold. | Stale sources stop dispatch and open exceptions; no optimistic sending. |
| Provider status semantics are inconsistent and out of order. | “Delivered” can be overstated. | Build recorded event sequences for every adapter and run fold property tests, duplicate replay, and reconciliation. | Only provider-defined terminal delivery maps to delivered; unknown states remain pending/exception. |
| Front desk staff may ignore another queue. | Proof without ownership does not reduce calls or no-shows. | Five-task usability test at 390 px and desktop; measure first exception assignment and resolution without coaching. | Median assignment <30 seconds, resolution <90 seconds, zero severe usability errors. |
| Minimal PHI may still be regulated data. | Breach impact and sales requirements are material. | Data inventory and threat model with an external security/privacy review; verify logs, exports, backups, and operator views using seeded canary strings. | No live patient data before critical findings are closed and BAA/DPA readiness is documented. |
| A single-process worker may fall behind. | Late reminders negate the product. | Load test 10× one target clinic’s peak, inject provider latency, and watch queue lag and lease recovery. | Split worker role or scale replicas before lag exceeds two minutes. |
| No-show reduction is hard to attribute. | Marketing could overstate impact. | Collect baseline and post-launch outcomes with sample size and seasonality notes; compare reminder-related subsets only. | Publish clinic-specific measured change, never a universal reduction claim. |
| Demo simulation may be mistaken for real provider proof. | Misleading evaluation damages trust. | Label every event “Simulated,” keep the banner persistent, and test that no provider origin is requested in the entire flow. | M1 review fails on any unlabeled simulated event or external provider request. |

## 7. Builder operating checklist

For every milestone:

1. Update its status to `in progress` before implementation and `complete` only after review/polish PASS.
2. Add new public claims to `.factory/claims.json`; each has exactly one tagged sandbox test.
3. Keep `?demo=1` working from a clean state and never let it touch production tenants or providers.
4. Use reversible migrations, tenant isolation tests, rate limits with `Retry-After`, structured redacted logs, and no required runtime environment except `PORT`.
5. Run `npm test`, `npm run check`, `npm run build`, all claim tests, browser/axe/link/console checks, and milestone-specific security/load tests.
6. Measure bundle and Lighthouse budgets rather than estimating them.
7. Update README, demo docs, copy audit, design/asset provenance, claims, this plan, and `.factory/handoff-m<N>.md`.
8. Commit a buildable tree. Do not deploy, change DNS, register billing, or send real messages unless that work order explicitly authorizes it.
