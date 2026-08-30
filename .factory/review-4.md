# Adversarial first-read review 4 — PASS

Date: 2026-08-30 UTC  
Live URL: <https://clinic-reminder-proof.sociobot.in>  
Reviewed repository commit: `759fb58c7476baa94dfc3a28b708f1be1871a245`  
Live footer build: `e40583f`

## Verdict

**PASS.** This review found zero blocking, medium, or minor findings. Every
declared claim was tested, the first-read gate passes on phone and desktop,
and every finding from reviews 1–3 remains fixed in the current live site and
source.

## Cold first read

Fresh Chromium contexts opened `/` at 390 × 844 and 1440 × 900 without
scrolling. There were no page errors or third-party requests.

| First-read question | Answer visible on the first screen |
| --- | --- |
| What does this do? | “See every reminder outcome.” |
| For whom? | “For independent clinics that need delivery proof and a clear next step when reminders fail.” |
| What should I click first? | “Try it with sample data,” beside “Opens a sample clinic. Nothing touches real clinic data.” |

The 390 px screen shows the headline, audience, primary action, action result,
and the three plain facts before the ledger preview. The action target is 44 px
high. This is a clear first action rather than a sign-in or sales gate.

## Copy audit

The line-by-line audit is [`.factory/copy-audit.md`](copy-audit.md). I checked
it against the current rendered landing page and current `README.md`; it lists
each landing sentence and each README prose sentence with its word count (27
landing entries across the first-screen and landing tables, and 87 README
entries; headings, actions, and facts are included, while commands are
excluded).

Results:

- No sentence exceeds 22 words.
- No banned marketing word, mood heading, or unexplained visitor-facing jargon
  remains.
- Headings name their sections: `Reminder evidence`, `How the sample clinic
  works`, `Limits and privacy`, and `Clinic plan price`.
- Buttons name their outcomes: `Try it with sample data`, `Advance due
  reminders`, `Resolve as Called patient`, `Reset demo`, and `Connect your
  clinic`.
- The consistent terms are `demo`, `messaging provider`, `reminder`, `attempt`,
  `exception`, `owner`, `source`, and `fallback`.
- Every landing or README promise that a visitor can rely on maps to a declared
  claim. No unlisted claim was found.

The wording corrected in the prior round remains present: `Build e40583f`,
`Page not found`, `Try the demo`, `A service on this site`, and
`stores each appointment once, even when it receives the same update twice`.

## Demo and sandbox

`/?demo=1` and `/demo` enter a populated Northline Sample Clinic in one click.
The first screen already shows five fictional appointments, outcomes, evidence
links, and Sofia R.'s staff-owned exception. The persistent banner reads
“Demo — sample data, nothing is saved to your clinic.” and includes `Reset
demo` and `Start for real`.

I advanced a reminder, assigned Sofia R. to Sam Rivera, and reset the demo.
The reset restored the original unassigned sample. Browser storage contained
only `demo:clinic-reminder-proof:<workspace>:active`; no non-demo key was
created. The complete request log contained only the product origin, and the
demo API routes were limited to `/api/v1/demo/...`. No messaging, clinic,
checkout, tracking, or third-party request occurred.

## Claims and local quality gates

All 31 exact commands in `.factory/claims.json` passed from fresh clone
`/tmp/clinic-reminder-proof-review4.71eViG`. The inherited shell had `CI=1`,
which makes Playwright start a competing server for each isolated command; the
commands were therefore run serially with only that inherited variable removed.
The manifest commands themselves were unchanged. The clean final rerun left no
failure artifacts.

`npm test` passed in the repository: 21 Vitest tests, 34 Rust tests, and 40
Chromium tests. `npm run check` passed with zero Svelte errors/warnings,
rustfmt clean, and Clippy warnings denied. `npm run build` produced `dist/`
and the release API binary. The public entry JavaScript is 28.63 KB gzip and
CSS is 5.54 KB gzip.

## History recheck

Every earlier `review-*.md`, `polish-*.md`, and handoff was read. Each earlier
finding was verified in both the live product and source, rather than accepted
from its closure note.

| Earlier finding | Current result |
| --- | --- |
| F-1-1 | The descriptive section headings remain live and in `App.svelte`. |
| F-1-2 | `sample-exception-visibility` is declared and passes through resolve/undo. |
| F-1-3–F-1-8 | The README repairs remain short, concrete, and use the documented terms. |
| F-2-1–F-2-5 | The signed intake, approved WhatsApp, callback verification, encryption, and data-minimisation claims all pass. |
| F-3-1 | The untestable originality wording remains removed; same-origin font loading is covered by `no-tracking`. |
| F-3-2 | The footer shows the live build prefix and `build-identity` passes. |
| F-3-3 | The real HTTP 404 uses the direct H1 `Page not found`. |
| F-3-4–F-3-8 | Demo and messaging-provider wording remains plain and consistent. |

## Structure, accessibility, and identity

- `/`, `/demo`, `/demo/reminders/mina`, `/start`, `/app`, `/privacy`, `/terms`,
  and `/404` have route titles, one H1, one main landmark, description,
  canonical URL, Open Graph title, and no browser error. An unknown route
  returns the designed page with HTTP 404.
- Header, skip link, navigation, legal footer, build identifier, back link,
  and focused route heading work across routes. All discovered internal links,
  the `#how` anchor, and the external Param Factory link returned successfully.
- `robots.txt` and `sitemap.xml` are present. The live response sends CSP with
  header-delivered `frame-ancestors 'none'`, HSTS, nosniff, and a strict-origin
  referrer policy.
- The browser suite covers keyboard, route focus/back behavior, 390 px and
  200% reflow, reduced motion, light/dark axe scans, and touch target size.
- The mineral evidence ledger, thin cyan event line, amber exception marker,
  and mono timestamp treatment visibly implement the approved translucent
  pulse-ledger direction. It is not a generic SaaS card/gradient layout.

## Missed leverage

No finding. The brief's expected high-value work is present: signed
calendar/EMR intake, consent-aware channel fallback, delivery receipts, a
staff exception queue, export/delete, and Sociobot-hosted billing. An AI step
is not implied by this reliability workflow and would be decorative.

## What would make this perfect

Maintain the present claim coverage and rerun the isolated demo and full-suite
checks after any change to reminder flow, privacy wording, routing, or build
identity. No additional product feature or copy change is indicated by this
review.
