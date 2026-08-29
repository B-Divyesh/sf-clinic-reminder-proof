# Independent product QA 8 — PASS

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-8`
Candidate: `b8ef87c632ac95f5ef2d41dec162cba7639eddd5`
Live URL: https://clinic-reminder-proof.sociobot.in

## Decision

**PASS.** The live deployment identifies itself as the candidate commit and
matches the locally built entry assets byte-for-byte. All declared claims,
the complete local suite, checks, production build, and independent live
product checks passed. No release-blocking defect was found.

## Mandatory first checks

`npm ci` completed from this candidate's lockfile (87 packages; audit reported
0 vulnerabilities), then every literal command in `.factory/claims.json` was
run individually through the demo entry point. The fail-fast command reached
its final `single-replica-durable-topology` command; the subsequent complete
suite independently ran every browser claim again.

| Claim | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS |
| `demo-cookie-lifetime` | PASS |
| `demo-replica-continuity` | PASS |
| `no-tracking` | PASS |
| `explicit-theme-choice` | PASS |
| `request-protection` | PASS |
| `rate-limit-policy` | PASS |
| `security-headers` | PASS |
| `build-identity` | PASS |
| `managed-auth-storage` | PASS (Rust and browser parts) |
| `no-marketing-campaigns` | PASS (Rust and browser parts) |
| `signed-in-export-delete` | PASS (Rust and browser parts) |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS |

Cold first-read, at 1440px, passes in plain words. The page says it lets
independent clinics “See every reminder outcome,” explains it provides
delivery proof and next steps when a reminder fails, and presents the visible
one-click **Try it with sample data** link beside “Opens a sample clinic.
Nothing touches real clinic data.” The link opens `/demo` and the persistent
banner says that sample data is not saved to the clinic.

## Local candidate verification

| Gate | Result |
| --- | --- |
| Clean source identity | PASS — `git rev-parse HEAD` was `b8ef87c632ac95f5ef2d41dec162cba7639eddd5`; worktree remained clean apart from ignored build/test outputs |
| `npm ci` | PASS |
| Exact individual claims commands | PASS — all 24 manifest entries (three include a paired Rust command) |
| `npm test` | PASS — 6 Vitest, 27 Rust, 31 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/warnings, `rustfmt --check`, Clippy with warnings denied |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` produced |
| Runtime defaults | PASS — optimized API started with only `PORT=18080`; it generated its local data key, served `/` and `/health`, and reported `build_sha: "dev"` as expected for a local build without a build arg |
| Container build | Not run — Docker is not installed in this verifier container; the repository production build and live container identity were verified instead |

The generated entry bundle is 81.35 KB raw / 28.34 KB gzip and CSS is 25.30
KB raw / 5.44 KB gzip. The 271.99 KB MSAL chunk is lazy. Self-hosted font
files total below the 120 KB budget. The initial application JS is therefore
well below the 200 KB static-product budget.

## Independent live evidence

- `GET /health` returned `200` and
  `{"status":"ok","build_sha":"b8ef87c632ac95f5ef2d41dec162cba7639eddd5"}`.
- SHA-256 of live and local `index-D8H9NAmY.js` matched
  `eb47f3560a3160b2556f836fb121b783e9efa3c740658d9b2967d4e0a022a404`;
  live and local `index-Uvd1xYxQ.css` matched
  `0930f7224ea1dc73363519a79439c4a3b9188fddfcbbe27cdc59b7a432f9e993`.
- In a fresh 390px browser context, `/demo` loaded five fictional appointments.
  Advancing due reminders yielded 4 due, 3 delivered with provider evidence,
  and 1 staff exception. Jordan L. showed simulated `TEMPLATE_REJECTED` on
  WhatsApp followed by simulated email fallback. Sofia R.'s opt-out created an
  exception without a provider attempt; assigning Sam Rivera, resolving,
  navigating/reloading, undoing, and reset all worked.
- Boundary probes on the live API returned structured JSON: wrong content type
  on checkout `415 content_type_invalid`; 17 KB checkout body
  `413 body_too_large`; unauthenticated export `401 bearer_required`; malformed
  demo JSON `400 json_invalid`. Each response carried a unique UUID that
  exactly matched `X-Request-Id`.
- Public routes `/`, `/demo`, `/start`, `/app`, `/privacy`, `/terms`, and
  `/404` returned 200 with route-specific title and one h1; an unknown route
  returned HTTP 404 with the styled recovery page.

## Privacy, security, accessibility, and browser checks

- A fresh landing-to-demo flow made only same-origin requests; no analytics,
  messaging-provider, billing, or account origin was contacted. The demo
  cookie claim passed with `HttpOnly`, `Secure`, `SameSite=Lax`, and
  `Max-Age=86400`.
- Fresh page loads had no page errors, console errors, or failed requests.
  `/opt/fleet/lib/verify-url.sh` also passed: title present, `lang=en`, one
  h1, main landmark, no missing image alt, no unlabeled button, and no console
  errors. Its evidence is in `/tmp/reminder-proof-qa8-verify/`.
- Independent axe scans of `/`, `/demo`, `/privacy`, and `/terms` at 390px
  reported zero serious or critical WCAG 2 A/AA findings.
- Keyboard-only testing exposed a 3px visible focus ring on the skip link;
  Enter moved focus to `<main>`. At 390px and 200% root text, `/demo` had no
  horizontal overflow. Reduced-motion demo rendering and the offline read-only
  state worked.
- HTML/API responses provide CSP, HSTS, `nosniff`, Referrer-Policy,
  Permissions-Policy and COOP. Root is `no-cache`; hashed JS is
  `public, max-age=31536000, immutable`.

## Backend, rate limit, and auth evidence

- Live demo creation allowed 5 requests for one supplied first-hop client
  address and then returned 429 on attempts 6 and 7 with `Retry-After: 3599`.
  This is the observed demo-create allowance: **5 per client per hour**.
- Additional single-client bursts demonstrated protection on non-health
  server endpoints: `/metrics` returned 429 with `Retry-After: 1` after its
  burst allowance, and unauthenticated checkout returned a mix of 401 and 429
  with `Retry-After: 1`. Health is the documented exemption.
- `/metrics` is machine-readable. `/api/v1/auth/config` returns only the
  required Sociobot Entra External ID tenant
  `sociobotcustomers.ciamlogin.com`, tenant ID
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, and expected client ID. Browser
  code uses MSAL PKCE with `sessionStorage`; no other sign-in provider was
  found.

## Defects by severity

- **Release-blocking / High / Medium:** None.
- **Low (non-blocking):** After deliberately exhausting the 5-per-hour demo
  creation allowance, Chromium logs its generic “Failed to load resource:
  429” console message even though the page catches it and shows the plain
  recovery message “Too many demo actions. Wait, then try again. Try again.”
  Fresh normal loads have zero errors. Consider suppressing the browser-level
  noise if a future implementation can do so without hiding the explicit 429
  recovery state.

## Known verification limits

No real Entra login, provider dispatch, purchase submission, or destructive
clinic deletion was attempted because no clinic identity, consented recipient,
provider credential, or payment authorization was supplied. The isolated
fixture/API and browser claim tests covering those paths passed. No code was
changed during this verification.
