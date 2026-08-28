# Reminder Proof visual thesis

Direction: **translucent pulse ledger**

Status: approved for M1 implementation

Last updated: 2026-08-28

## Thesis

Reminder Proof should feel like a live evidence book on a quiet clinic desk: exact enough to trust, calm enough to scan under pressure, and luminous only where new evidence arrives.

Thin translucent planes represent source, consent, attempt, provider event, response, and staff resolution. They align into one horizontal ledger so the causal chain is visible. A single cyan pulse marks the newest event, then becomes a static dot. Warm amber breaks the cool system when a person must act. This makes the visual treatment explanatory rather than decorative.

The product must not resemble a generic blue healthcare dashboard. Avoid pill-filled card grids, gradient blobs, stock clinician photography, oversized metric tiles, and glass effects without information hierarchy. Translucency is reserved for overlapping evidence and drawers. The main reading plane stays nearly opaque.

## Why this fits the product

- A ledger is durable evidence, not a vague activity feed.
- A pulse implies a delivery event without suggesting guaranteed patient attention.
- Layering makes provenance visible: the calendar said one thing, consent allowed another, and the provider returned a third.
- The cool mineral palette stays calm; amber and rose make exceptions legible without alarm fatigue.
- The system works at front-desk density and on a 390 px phone because the identity lives in line, light, and sequence rather than large artwork.

## Stack and rendering decision

Use Svelte 5 + Vite + strict TypeScript for the web app. The ledger, evidence drawer, filters, and exception assignment have meaningful reactive state, while Svelte avoids the runtime and library weight of a React dashboard. Use native HTML controls and platform CSS wherever possible. Use Rust/axum for the same-origin API and static serving. This is an operational product, so no WebGL, canvas, runtime image generation, or decorative animation library belongs in the initial build.

## Palette

The default is a light “clinic daylight” treatment. A complete dark “after-hours” treatment follows the user’s system preference and can be chosen explicitly. Both use the same semantic roles.

| Token | Light | Dark | Purpose |
| --- | --- | --- | --- |
| `--color-bg` | `#f3f7f5` | `#071519` | Page ground |
| `--color-surface` | `#fbfdfc` | `#0d2329` | Primary opaque reading surface |
| `--color-surface-raised` | `#ffffff` | `#123139` | Drawer and floating controls |
| `--color-glass` | `rgba(251,253,252,.78)` | `rgba(13,35,41,.82)` | Provenance overlays only |
| `--color-text` | `#102a33` | `#e7f5f2` | Primary text |
| `--color-muted` | `#48636b` | `#aac3c5` | Secondary text; verified ≥4.5:1 |
| `--color-border` | `#a9c1c1` | `#36535a` | Dividers and control edges |
| `--color-accent` | `#006a73` | `#63d8d3` | Links, primary action, current event |
| `--color-accent-contrast` | `#ffffff` | `#062025` | Text on accent |
| `--color-success` | `#216e4a` | `#6bddaa` | Delivered/complete |
| `--color-warning` | `#8a4f00` | `#f4bb62` | Needs staff action |
| `--color-danger` | `#a12c3f` | `#ff8997` | Failed, blocked, destructive |
| `--color-reply` | `#5b4ab4` | `#b9a9ff` | Patient reply |
| `--color-focus` | `#005fcc` | `#8ac7ff` | Focus ring, distinct from status |

Use status color only with a text label and a shape: check for delivered, square stop for blocked, diamond for exception, arrow for fallback, speech mark for response. Do not tint large backgrounds red or green. Table and timeline numbers use tabular figures.

Contrast must be tested from the executable token file in both themes. Body text is at least 4.5:1; focus, borders, and meaningful graphical objects are at least 3:1 against adjacent colors.

## Typography

- **Interface and display:** Instrument Sans, self-hosted OFL, weights 500 and 650 as one variable WOFF2 subset.
- **Evidence and numbers:** Fragment Mono, self-hosted OFL, weight 400 as one WOFF2 subset.
- **Fallback:** `Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif` for interface and `ui-monospace, "SFMono-Regular", Consolas, monospace` for evidence until M1 adds the licensed files.

Instrument Sans is compact without feeling institutional. Fragment Mono separates timestamps, channel IDs, and provider codes from prose. No patient-facing text uses the mono face.

| Token | Size / line | Use |
| --- | --- | --- |
| `--text-xs` | 14 / 20 px | Metadata only; never the main instruction |
| `--text-sm` | 16 / 24 px | Compact controls and table text |
| `--text-md` | 20 / 30 px | Lead and section intro |
| `--text-lg` | 25 / 32 px | Panel heading |
| `--text-xl` | 32 / 38 px | Page heading |
| `--text-display` | 48 / 52 px, 40 / 44 px below 768 px | Landing headline only |

Body copy is 16 px minimum, with a 60–70 character measure. Use sentence case. Button labels start with verbs. “Proof” means evidence, “attempt” means one provider request, and “exception” means a staff-owned task; do not substitute synonyms.

## Spacing, shape, depth

Use a 4 px base and an 8 px layout rhythm:

- `--space-1` 4 px
- `--space-2` 8 px
- `--space-3` 12 px
- `--space-4` 16 px
- `--space-5` 24 px
- `--space-6` 32 px
- `--space-7` 48 px
- `--space-8` 64 px
- `--space-9` 96 px

Control height is 44 px minimum. Dense ledger rows are 64 px collapsed and expand naturally. Adjacent targets keep at least 8 px of separation.

- `--radius-sm` 6 px for chips and inset evidence.
- `--radius-md` 12 px for controls and rows.
- `--radius-lg` 20 px for drawers and major independent surfaces.
- Buttons are clipped rectangles, not capsules.
- Ledger surfaces use a 1 px border and one slate-tinted shadow. Avoid nesting more than two bordered surfaces.
- Glass uses `backdrop-filter` only as progressive enhancement. The fallback is an opaque surface with the same contrast.

Group by whitespace before adding a card. The landing preview, ledger, and exception drawer are independent surfaces; headings and explanatory copy are not cards.

## Layout rhythm

The landing page is editorial and asymmetric, not a centered hero. From 1024 px, copy occupies five columns and the clipped working ledger occupies seven. The first evidence line begins above the headline baseline, suggesting that the system is already monitoring. At 390 px, copy, action, three facts, and then ledger preview form one direct column.

The application uses a stable top bar, a narrow date/status rail, a flexible ledger, and an optional evidence drawer. The primary task always owns the widest region. The exception queue favors list density over dashboards: one clear summary line, then rows.

Maximum shell width is 1440 px. Long-form legal and help content is 720 px. Page blocks follow 64 px mobile / 96 px desktop vertical rhythm on marketing pages and 24–32 px rhythm in the app.

## Interaction grammar

- **Inspect:** selecting an attempt expands its evidence from the originating row. On desktop the drawer enters from the row edge; on mobile it becomes a labeled modal sheet.
- **Advance:** new evidence draws one thin accent line from the previous node to the new node, then settles.
- **Own:** assigning an exception moves the owner label into place and announces the change. The row does not jump between sections until focus leaves it.
- **Resolve:** show the exact reminder, resolution code, and consequence. Offer an inline undo when dispatch safety permits.
- **Filter:** update results immediately, announce the count after a short debounce, preserve filters in the URL, and keep the heading stable.
- **Navigate:** real links update history and titles. Route changes focus the new `<h1>`; back/forward restores route, scroll, and useful focus.

Every action has pressed, busy, success, and error feedback. Provider calls and server writes are never optimistically called complete. Links remain visibly links; buttons perform actions.

## Motion policy

Motion communicates new evidence and spatial continuity only.

- Micro state: 160 ms ease-out.
- Drawer and route region: 220 ms cubic-bezier(.2,.8,.2,1).
- New-event pulse: one 600 ms line-and-dot sequence, never looping.
- Maximum ambient drift in the landing ledger: 12 px over 10 seconds, only while visible, pausable, and omitted on small screens.
- Animate only transform and opacity. Reserve dimensions before transitions.
- Under `prefers-reduced-motion: reduce`, remove travel, parallax, smooth scroll, and skeleton shimmer. Use an immediate state change plus a 120 ms opacity cue at most.
- Never flash, oscillate, or autoplay sound/video.

## Component inventory

The machine-readable inventory is `packages/design-system/component-inventory.json`. These are the implementation notes.

1. **App shell** — header, main landmark, optional rail/drawer, safe-area padding. States: public, demo, signed-in, offline.
2. **Site header** — wordmark home link and at most four navigation links. States: wide, compact, signed-in menu.
3. **Demo banner** — persistent “Demo — sample data, nothing is saved to your clinic,” Reset demo, Start for real. States: active, resetting, reset failed.
4. **Route announcer** — visually hidden polite region. States: idle, announcing.
5. **Primary button** — one per decision region. States: default, hover, pressed, focus-visible, busy, disabled.
6. **Secondary button** — non-primary or reversible action. Same states; never lower than 3:1 boundary contrast.
7. **Text link** — underlined or arrow-afforded, with external link label where needed.
8. **Field** — label, control, help, error, required explanation. States: default, focus, filled, invalid, disabled, read-only.
9. **Status chip** — icon + stable word. States: scheduled, attempted, delivered, replied, blocked, exception, resolved, simulated.
10. **Consent badge** — channel, allowed/blocked/unknown, source, captured time. Unknown is never treated as allowed.
11. **Filter bar** — date, location, channel, status, owner, search, clear. Collapses to a labeled sheet on mobile.
12. **Ledger timeline** — ordered evidence chain with text alternative. States: compact, expanded, pending, complete, exception.
13. **Attempt row** — time, channel, exact state, provider code summary. States: selected, new, stale, loading detail.
14. **Pulse marker** — shape and label at a timeline point. Motion occurs once; reduced mode is static.
15. **Exception card** — reason, appointment time, safe next action, age, owner. States: unassigned, assigned to me, assigned elsewhere, overdue, resolved.
16. **Assignment control** — native select/combobox depending list size. States: idle, saving, conflict, saved, error.
17. **Confirmation dialog** — destructive or dispatch-affecting actions only. Traps and restores focus, names the affected record.
18. **Inline notice / toast** — notices stay near the action; toast supplements but never replaces persisted status. States: info, success, warning, danger.
19. **State panel** — shared geometry for empty, loading, error, expired demo, and offline. Each has one useful next step.
20. **Evidence drawer** — source, consent, policy, attempts, responses, exception, audit link. States: loading, ready, partial provider data, error; full-screen dialog on mobile.

## Key screens

### 1. Landing and preview

Headline: “See every reminder outcome.” Supporting sentence names independent clinics and failed-reminder follow-up. The sample action is adjacent to the plain result “Opens a sample clinic. Nothing touches real clinic data.” Three facts follow as short lines. The ledger preview is live sample UI, clipped by the viewport rather than framed as a laptop mockup. It includes one delivered line, one fallback, and one owned exception with visible “Simulated” labels.

### 2. Today’s delivery ledger

The `<h1>` states the date in words. A compact summary gives due, delivered, pending, and exceptions with denominators. Rows lead with appointment time and patient alias, then channels and exact outcome. Expansion reveals the evidence in place. The primary action is “Review exceptions” only when exceptions exist; otherwise it is “Check source health.”

### 3. Reminder evidence

The evidence drawer reads as a causal chain: source appointment → consent snapshot → policy version → attempt → provider event → response → staff resolution. Unknown and missing events occupy explicit gaps. Raw provider codes live behind “Show technical detail.” The public demo uses fictional aliases and labels each event simulated.

### 4. Exception queue

Rows sort by appointment risk, not by technical error. The reason is plain: “SMS provider rejected this number. Email is not allowed.” Assignment is inline. Resolution offers exact choices such as “Called patient,” “Corrected contact,” “Appointment cancelled,” or “No safe channel.” Free notes are optional, short, and explicitly non-clinical.

### 5. Setup and dry run

A left step list shows Source, Consent, Channels, Policy, and Dry run. Each completed step displays its evidence, not a decorative check alone. The dry run uses upcoming appointments and shows send/block/exception counts without contacting a provider. Activation repeats the consequences and requires an owner to confirm.

## Responsive behavior

- At 390 px, drop the persistent side rail and multi-column summaries. Keep outcome words, owner, and next action. Put technical codes under disclosure.
- Tables become semantic lists only when column relationships are retained in accessible names. Do not force horizontal scroll for core actions.
- The evidence drawer is a full-screen dialog on phones, a 420–520 px side drawer on tablets, and an optional fixed second pane on wide screens.
- Sticky controls account for safe-area insets and never cover focused inputs or the final row.
- At 200% zoom, regions reflow to one column and retain every control and label.
- Hover reveals nothing essential. Touch and keyboard paths expose the same evidence.

## State copy principles

- Empty ledger: “No reminders are due in this range.” Action: “Choose another date.”
- Unsynced source: “This source has not synced yet. Check the connection before reminders can run.” Action: “Check source.”
- Provider pending: “The provider accepted this attempt. Delivery is not confirmed yet.”
- Exhausted: “No allowed channel delivered this reminder. Assign someone to follow up.”
- Offline: “You’re offline. This ledger was last updated at {time}. Sending and resolving are unavailable.”
- Error: name the failed step, say whether scheduled reminders are affected, and give one recovery action plus a request ID.
- Never use “success” for accepted/queued, “failed” for unknown, or “patient read” unless the provider supplies a documented read event.

## Accessibility contract

- Exactly one `<h1>` per route, heading levels in order, and `header`, `nav`, `main`, and `footer` landmarks.
- A first-focus skip link and a persistent polite route announcement.
- Native buttons, links, tables, inputs, selects, and dialogs before custom widgets.
- Visible focus ring at least 3 CSS px with 3:1 contrast and a 2 px offset.
- Controls are at least 44×44 CSS px with 8 px separation.
- Text contrast is at least 4.5:1; UI and status shapes at least 3:1 in both themes.
- Status always combines word, icon/shape, and position; timelines include a linear text alternative.
- Dialogs name themselves, trap focus, close with Escape when safe, and return focus to their trigger.
- Async changes announce concise results; repeated provider updates are batched to avoid screen-reader noise.
- Reduced-motion, forced-colors, 200% text zoom, keyboard-only, and screen-reader smoke checks are release gates.

## Route titles and metadata

| Route | Title | H1 |
| --- | --- | --- |
| `/` | `Reminder Proof — See every reminder outcome` | See every reminder outcome |
| `/demo` | `Demo — Reminder Proof` | Today’s sample reminders |
| `/demo/reminders/:id` | `Reminder evidence — Reminder Proof` | Evidence for {time} appointment |
| `/privacy` | `Privacy — Reminder Proof` | How Reminder Proof handles data |
| `/terms` | `Terms — Reminder Proof` | Terms for Reminder Proof |
| `/404` | `Page not found — Reminder Proof` | This page has no ledger entry |
| `/app/ledger` | `Delivery ledger — Reminder Proof` | Delivery ledger for {date} |
| `/app/exceptions` | `Exceptions — Reminder Proof` | Reminders that need a person |
| `/app/setup/*` | `{Step} setup — Reminder Proof` | Set up {step} |
| `/app/settings/billing` | `Plan and billing — Reminder Proof` | Plan and billing |

Default description: “Track appointment reminder attempts, delivery evidence, safe fallbacks, and staff-owned exceptions without replacing your clinic calendar.” (147 characters.) Canonical and social metadata use the production origin. M1 must add a real 1200×630 social image derived from the ledger art, an SVG favicon, a 180 px touch icon, sitemap, robots, and per-route metadata.

## Original asset plan and provenance

M1 asset implementation record (2026-08-28): no generated imagery or stock material was used. All visual assets below are original repository assets or self-hosted OFL font files emitted by the build.

M1 creates these original assets:

1. **Pulse ledger field** — created as `apps/web/src/lib/art/PulseLedger.svelte`. It is hand-authored SVG/CSS with parallel ruled lines, translucent evidence slips, five semantic markers, and no embedded text. License: repository MIT.
2. **Social card** — created as `apps/web/public/social-card.svg`, 1200×630, composed locally from the pulse-ledger field and live typography. No stock material. License: repository MIT.
3. **Favicon/touch icon** — created as `apps/web/public/favicon.svg` and `apps/web/public/apple-touch-icon.png`, hand-authored ledger lines plus proof dot. License: repository MIT.
4. **Instrument Sans and Fragment Mono subsets** — self-hosted through the official `@fontsource-variable/instrument-sans` 5.3.0 and `@fontsource/fragment-mono` 5.2.6 packages. Vite emits the used local assets with `font-display: swap`; OFL source URLs and licenses are recorded in `apps/web/public/fonts/README.md`.

If image generation is introduced later, record the exact prompt, model/deployment, date, edits, review for medical symbols/text/artifacts, and output license here. Generated imagery must be disclosed in the footer/about page. M1 does not need generated raster art; the code-drawn evidence field is more faithful and lighter.

## Performance budget

- Initial JS ≤150 KB gzip for public routes and ≤200 KB for authenticated app routes.
- CSS ≤50 KB gzip.
- Font WOFF2 total ≤120 KB, maximum two preloads.
- Social/hero raster, if any, ≤300 KB at mobile size with dimensions reserved.
- LCP <2.5 seconds, INP <200 ms, CLS <0.1 on a throttled mid-range phone.
- Lighthouse mobile ≥90 performance and ≥95 accessibility.
- Target ES2022, tree-shaken imports, route-level code splitting, no full utility libraries, no runtime CDN.

## Content vocabulary

| Concept | Always say | Do not alternate with |
| --- | --- | --- |
| Scheduled communication | reminder | notification, campaign, outreach |
| One provider request | attempt | send, ping |
| Evidence chain | timeline | activity feed, history log |
| Provider-confirmed result | delivered | successful, complete |
| Human task | exception | issue, incident, ticket |
| Responsible staff member | owner | assignee, handler |
| Source calendar/EMR import | source | integration feed, upstream |
| Ordered alternate channels | fallback | reroute, failover |

All interface copy follows `.factory/copy-audit.md` and the plain-words limits. Marketing may state tested product behavior only when the matching claim exists and passes.
