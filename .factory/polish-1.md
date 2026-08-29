# Polish round 1 evidence

Date: 2026-08-29 UTC

Review commit: `ab7ae213b8e77b8b5ab56145d373973f5e7a7cb0`

Runtime repair commit: `c8e6746c675d0d0d2f9ba42604734bb2f3c754fb`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Finding closure

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | Replaced slogan-like landing labels with `Reminder evidence`, `Limits and privacy`, and `Clinic plan price`. Kept the useful reminder-evidence sentence as supporting copy. | Playwright `landing sections use descriptive headings and the first screen names the job, audience, and next step`; live `/`; `.factory/qa-artifacts/polish-1/live/screenshot-desktop.png` |
| F-1-2 | Added `sample-exception-visibility` to `.factory/claims.json`. The demo now distinguishes open work from recently resolved evidence, marks the ledger row `Resolved`, preserves reload state, and retains Undo. | `npm run test:e2e -- --grep @claim:sample-exception-visibility`; passed locally, from the clean clone, and live at `/?demo=1`; `.factory/qa-artifacts/polish-1/live/resolved-mobile.png` |
| F-1-3 | Split the 31-word README opening into two concrete sentences of 9 and 10 words. | Vitest `README plain-words repairs keep each reviewed sentence short and concrete`; `.factory/copy-audit.md` |
| F-1-4 | Split the 33-word README demo list into two sentences of 7 and 12 words. | Same Vitest copy regression and copy audit |
| F-1-5 | Split the 27-word managed-workflow sentence into consent and receipt/exception sentences of 7 and 13 words. | Same Vitest copy regression and copy audit |
| F-1-6 | Replaced the jargon-heavy API bullet with two plain sentences about protection, limits, health, and metrics. | Same Vitest copy regression and copy audit |
| F-1-7 | Split durable writes and daily recovery into sentences of 12 and 11 words. | Same Vitest copy regression and copy audit |
| F-1-8 | Split storage mounts and process privilege into sentences of 11 and 6 words; kept SMB rationale in the operations guide. | Same Vitest copy regression and copy audit |

## Cumulative acceptance re-check

| Area | Evidence |
| --- | --- |
| First screen | The desktop and 390 px cold views state the job, independent-clinic audience, sample action, next result, privacy, content, and price. Browser heading/copy test passed live. |
| One-click isolated demo | Both `/?demo=1` and `/demo` open five fictional appointments with the persistent banner, Reset demo, and Start for real. `demo-isolation`, `demo-reset`, and `no-tracking` passed live. |
| Claims | All 25 exact manifest commands passed independently from `/tmp/clinic-reminder-proof-polish-1.dltz6U`; the full suite repeated every browser claim. |
| Titles, metadata, routing, focus, 404, legal links | `explicit-theme-choice`, `public pages have no console errors and local links resolve`, `keyboard, mobile, deep links, back navigation, and offline reads work`, and `unknown browser routes return an HTTP 404` passed live. `/privacy` and `/terms` returned 200. |
| Mobile and accessibility | 390 × 844, 200% text, keyboard-only, reduced-motion, focus restoration, and light/dark axe checks passed. Live Lighthouse accessibility was 100. |
| Privacy and offline | Whole landing-to-demo request logging remained same-origin. Offline cached demo reads and disabled writes passed. No analytics or third-party runtime request loaded. |
| Performance | Public JS is 28.58 KB gzip and CSS is 5.52 KB gzip. Live mobile Lighthouse: Performance 100, LCP 1.28 s, CLS 0, TBT 52 ms. |
| Backend and deployment | 27 Rust tests passed. Live health reports the repair SHA. Revision `0000028` has one replica, both durable mounts, and 100% traffic. |

## Visual evidence

- Local landing: `.factory/qa-artifacts/polish-1/local-landing-desktop.png`
- Local demo at 390 px: `.factory/qa-artifacts/polish-1/local-demo-mobile.png`
- Local resolved lifecycle at 390 px: `.factory/qa-artifacts/polish-1/local-resolved-mobile.png`
- Live cold landing: `.factory/qa-artifacts/polish-1/live/screenshot-desktop.png`
- Live cold landing at 390 px: `.factory/qa-artifacts/polish-1/live/screenshot-mobile.png`
- Live resolved lifecycle at 390 px: `.factory/qa-artifacts/polish-1/live/resolved-mobile.png`

No review finding remains open.
