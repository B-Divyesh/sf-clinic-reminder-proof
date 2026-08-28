import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

let demoClient = 10;

test.beforeEach(async ({ page }, testInfo) => {
  demoClient += 1;
  await page.context().setExtraHTTPHeaders({
    'x-forwarded-for': `198.18.${testInfo.workerIndex}.${demoClient}`
  });
});

async function openDemo(page: import('@playwright/test').Page) {
  await page.goto('/?demo=1');
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
}

test('@claim:demo-isolation Demo actions use sample data only and never contact a messaging provider.', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await openDemo(page);
  await page.getByRole('button', { name: 'Advance due reminders' }).click();
  await page.getByLabel('Owner for Sofia R.').selectOption({ label: 'Sam Rivera' });
  await page.getByRole('button', { name: 'Resolve as Called patient' }).click();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('The original sample clinic is restored.')).toBeVisible();
  const origin = new URL(page.url()).origin;
  expect(requests.every((url) => new URL(url).origin === origin)).toBe(true);
  expect(requests.some((url) => url.includes('/api/v1/demo/'))).toBe(true);
  expect(requests.every((url) => !/twilio|whatsapp|dodo|checkout/i.test(url))).toBe(true);
});

test('@claim:sample-outcome-coverage Every due sample reminder has delivery evidence or a staff-owned exception.', async ({ page }) => {
  await openDemo(page);
  await page.getByRole('button', { name: 'Advance due reminders' }).click();
  await expect(page.locator('.summary-grid')).toContainText('Due4');
  await expect(page.locator('.summary-grid')).toContainText('Delivered3');
  await expect(page.locator('.summary-grid')).toContainText('Exceptions1');
  const dueRows = page.locator('.ledger-list li').filter({ hasText: 'Today' });
  await expect(dueRows).toHaveCount(4);
  await expect(dueRows.filter({ hasText: 'Scheduled' })).toHaveCount(0);
  await expect(dueRows.filter({ hasText: /Delivered|Needs owner/ })).toHaveCount(4);
});

test('@claim:consent-channel-guard A sample channel without recorded consent is blocked before dispatch.', async ({ page }) => {
  await openDemo(page);
  await page.getByRole('link', { name: /View evidence for Sofia R/ }).click();
  await expect(page.getByRole('heading', { level: 1 })).toContainText('Evidence for Today, 14:00 appointment');
  await expect(page.locator('.timeline-panel')).toContainText('SMS is blocked by an opt-out. No provider attempt was made.');
  await expect(page.locator('.timeline-panel')).not.toContainText('Provider result');
  await expect(page.locator('.detail-exception')).toContainText('Assign someone to follow up.');
});

test('@claim:fallback-order A simulated delivery failure tries the next allowed sample channel.', async ({ page }) => {
  await openDemo(page);
  await page.getByRole('button', { name: 'Advance due reminders' }).click();
  await page.getByRole('link', { name: /View evidence for Jordan L/ }).click();
  const timeline = page.locator('.timeline-panel');
  await expect(timeline).toContainText('WhatsApp');
  await expect(timeline).toContainText('TEMPLATE_REJECTED');
  await expect(timeline).toContainText('Email fallback accepted by the simulated provider.');
  await expect(timeline).toContainText('Provider modeSimulated');
  const attempts = timeline.locator('li').filter({ hasText: /WhatsApp|Email/ });
  await expect(attempts).toHaveCount(3);
});

test('@claim:delivery-timeline The sample timeline shows the channel, time, provider result, and exact outcome for each attempt.', async ({ page }) => {
  await openDemo(page);
  await page.getByRole('button', { name: 'Advance due reminders' }).click();
  await page.getByRole('link', { name: /View evidence for Mina P/ }).click();
  const timeline = page.locator('.timeline-panel');
  await expect(timeline).toContainText('08:01');
  await expect(timeline).toContainText('ChannelSMS');
  await expect(timeline).toContainText('Provider resultDELIVERED-200');
  await expect(timeline).toContainText('OutcomeDelivered');
  await expect(timeline).toContainText('Provider modeSimulated');
});

test('@claim:exception-ownership Staff can assign and resolve a sample exception.', async ({ page }) => {
  await openDemo(page);
  await page.getByLabel('Owner for Sofia R.').selectOption({ label: 'Sam Rivera' });
  await expect(page.getByText('Owner saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Resolve as Called patient' }).click();
  await expect(page.locator('.exception-row')).toContainText('Resolved: Called patient');
  await page.reload();
  await expect(page.locator('.exception-row')).toContainText('Sam Rivera');
  await expect(page.locator('.exception-row')).toContainText('Resolved: Called patient');
  await page.getByRole('button', { name: 'Undo resolution' }).click();
  await expect(page.locator('.exception-row')).toContainText('Resolve as Called patient');
});

test('@claim:demo-reset Reset demo restores the original sample clinic.', async ({ page }) => {
  await openDemo(page);
  await page.getByLabel('Owner for Sofia R.').selectOption({ label: 'Sam Rivera' });
  await page.getByRole('button', { name: 'Resolve as Called patient' }).click();
  await page.getByRole('button', { name: 'Advance due reminders' }).click();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('.summary-grid')).toContainText('Due4');
  await expect(page.locator('.summary-grid')).toContainText('Delivered1');
  await expect(page.locator('.summary-grid')).toContainText('Exceptions1');
  await expect(page.getByLabel('Owner for Sofia R.')).toHaveValue('');
  await expect(page.getByRole('button', { name: 'Resolve as Called patient' })).toBeDisabled();
});

test('@claim:minimal-reminder-content Sample reminder contents exclude clinical notes, diagnoses, and treatment details.', async ({ page }) => {
  await openDemo(page);
  const data = await page.evaluate(async () => (await fetch('/api/v1/demo/state')).text());
  expect(data).toContain('Mina P.');
  expect(data).toContain('Northline Sample Clinic');
  expect(data).not.toMatch(/diagnos|clinical note|treatment|date of birth|insurance|address|phone|email address/i);
});

test('@claim:public-price The Clinic plan costs $79 per location each month, plus published messaging charges.', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('See every reminder outcome.');
  await expect(page.locator('.pricing-section')).toContainText('$79');
  await expect(page.locator('.pricing-section')).toContainText('per location each month');
  await expect(page.locator('.pricing-section')).toContainText('published messaging charges');
  await page.getByRole('link', { name: 'Terms' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Terms for Reminder Proof');
  await expect(page.locator('.legal-page')).toContainText('$79 per location each month, plus published messaging charges.');
});

test('@claim:demo-cookie-lifetime Demo state uses an isolated HttpOnly Secure cookie that expires within 24 hours.', async ({ page }) => {
  const response = await page.request.post('/api/v1/demo/workspaces');
  expect(response.status()).toBe(200);
  const cookie = response.headers()['set-cookie'];
  expect(cookie).toContain('HttpOnly');
  expect(cookie).toContain('Secure');
  expect(cookie).toContain('SameSite=Lax');
  expect(cookie).toContain('Max-Age=86400');
});

test('@claim:demo-replica-continuity Demo changes remain after navigation, reload, and repeated state reads.', async ({ page }) => {
  await openDemo(page);
  await page.getByLabel('Owner for Sofia R.').selectOption({ label: 'Sam Rivera' });
  const first = await page.evaluate(async () => (await fetch('/api/v1/demo/state')).json());
  for (let read = 0; read < 30; read += 1) {
    const result = await page.evaluate(async () => {
      const response = await fetch('/api/v1/demo/state');
      return { status: response.status, state: await response.json() };
    });
    expect(result.status).toBe(200);
    const state = result.state;
    expect(state.demo.workspace_id).toBe(first.demo.workspace_id);
    expect(JSON.stringify(state)).toContain('Sam Rivera');
  }
  await page.reload();
  await expect(page.getByLabel('Owner for Sofia R.')).toHaveValue('Sam Rivera');
});

test('@claim:no-tracking No tracking script or third-party runtime request loads.', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/');
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
  const origin = new URL(page.url()).origin;
  expect(requests.length).toBeGreaterThan(2);
  expect(requests.every((url) => new URL(url).origin === origin)).toBe(true);
});

test('@claim:request-protection API writes enforce JSON and 16 KB body limits with structured errors.', async ({ page }) => {
  const malformed = await page.request.post('/api/v1/demo/exceptions/sofia-exception/assign', {
    headers: { 'content-type': 'application/json' },
    data: Buffer.from('{')
  });
  expect(malformed.status()).toBe(400);
  expect(await malformed.json()).toMatchObject({ code: 'json_invalid' });
  const tooLarge = await page.request.post('/api/v1/demo/exceptions/sofia-exception/assign', {
    headers: { 'content-type': 'application/json' },
    data: Buffer.from('x'.repeat(17_000))
  });
  expect(tooLarge.status()).toBe(413);
  expect(await tooLarge.json()).toMatchObject({ code: 'body_too_large' });
});

test('@claim:rate-limit-policy Demo creation is limited by the ingress client address and returns Retry-After.', async ({ page }) => {
  let last: import('@playwright/test').APIResponse | undefined;
  for (let request = 0; request < 6; request += 1) {
    last = await page.request.post('/api/v1/demo/workspaces', {
      headers: { 'x-forwarded-for': `192.0.2.${request}, 203.0.113.220` }
    });
  }
  expect(last?.status()).toBe(429);
  expect(Number(last?.headers()['retry-after'])).toBeGreaterThan(0);
  expect(await last?.json()).toMatchObject({ code: 'rate_limited' });
});

test('@claim:security-headers Responses use the documented browser security and cache headers.', async ({ page }) => {
  const pageResponse = await page.request.get('/');
  expect(pageResponse.headers()['content-security-policy']).toContain("default-src 'self'");
  expect(pageResponse.headers()['strict-transport-security']).toContain('max-age=31536000');
  expect(pageResponse.headers()['x-content-type-options']).toBe('nosniff');
  await page.goto('/');
  const assetPath = await page.locator('script[type="module"]').getAttribute('src');
  const assetResponse = await page.request.get(assetPath!);
  expect(assetResponse.headers()['cache-control']).toContain('immutable');
});

test('@claim:build-identity Health reports the running build identity and metrics are machine-readable.', async ({ page }) => {
  const health = await page.request.get('/health');
  expect(health.status()).toBe(200);
  expect(await health.json()).toMatchObject({ status: 'ok' });
  const metrics = await page.request.get('/metrics', {
    headers: { 'x-forwarded-for': `198.18.200.${demoClient}` }
  });
  expect(metrics.status()).toBe(200);
  expect(await metrics.text()).toContain('reminder_proof_http_requests_total');
});

test('@claim:managed-auth-storage Clinics use protected Entra-managed workspace entry points.', async ({ page }) => {
  await page.goto('/start');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Connect your clinic reminders');
  await expect(page.getByText('Sociobot Microsoft Entra protects each clinic workspace.')).toBeVisible();
  await expect(page.getByText('A signed calendar feed and encrypted provider credentials keep systems separate.')).toBeVisible();
  await expect(page.getByText('Consent, fallback attempts, signed receipts, and staff action stay in one timeline.')).toBeVisible();
  await expect(page.locator('input[type="file"]')).toHaveCount(0);
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('real:')))).toEqual([]);
  await expect(page.getByText('Subscription checkout opens after you sign in and create a clinic workspace.')).toBeVisible();
  const config = await page.request.get('/api/v1/auth/config');
  expect(await config.json()).toMatchObject({ tenant_id: '35c6fe40-0ec0-46b6-98c6-213ad4de6650', client_id: '25c704f4-465a-47af-80ab-2c489466b697', authority: 'https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/' });
  for (const endpoint of ['/api/v1/clinic', '/api/v1/clinic/export']) {
    const response = await page.request.get(endpoint, { maxRedirects: 0 });
    expect(response.status(), endpoint).toBe(401);
    expect(response.headers()['www-authenticate']).toBe('Bearer');
  }
  const checkout = await page.request.post('/api/v1/billing/checkout', { data: { tier: 'clinic' } });
  expect(checkout.status()).toBe(401);
  expect(checkout.headers()['www-authenticate']).toBe('Bearer');
  const intake = await page.request.post('/api/v1/connectors/intake', { data: { connector_id: 'missing', appointments: [] } });
  expect(intake.status()).toBe(404);
  await page.goto('/app');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Clinic reminder ledger');
  await expect(page.getByRole('button', { name: 'Sign in with Microsoft' })).toBeVisible();
});

test('public routes have no serious or critical axe findings', async ({ page }) => {
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme });
    for (const path of ['/', '/demo', '/start', '/app', '/privacy', '/terms', '/missing']) {
      await page.goto(path);
      if (path === '/demo') await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
      const results = await new AxeBuilder({ page }).analyze();
      expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''))).toEqual([]);
    }
  }
});

test('public pages have no console errors and local links resolve', async ({ page }) => {
  const consoleErrors: string[] = [];
  const failedRequests: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });
  page.on('requestfailed', (request) => failedRequests.push(`${request.method()} ${request.url()}`));
  const localLinks = new Set<string>();
  for (const path of ['/', '/demo', '/start', '/privacy', '/terms', '/404']) {
    await page.goto(path);
    if (path === '/demo') await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
    if (path === '/start') await expect(page.getByRole('button', { name: 'Sign in with Microsoft' })).toBeEnabled();
    for (const href of await page.locator('a[href]').evaluateAll((links) => links.map((link) => link.getAttribute('href') ?? ''))) {
      if (href.startsWith('/')) localLinks.add(href);
    }
  }
  for (const href of localLinks) {
    const response = await page.request.get(href);
    expect(response.status(), href).toBe(200);
  }
  expect(consoleErrors).toEqual([]);
  expect(failedRequests).toEqual([]);
});

test('keyboard, mobile, deep links, back navigation, and offline reads work', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openDemo(page);
  await page.getByRole('link', { name: /View evidence for Eli K/ }).click();
  await expect(page.getByRole('heading', { level: 1 })).toContainText('Evidence for Today, 15:30 appointment');
  await page.goBack();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
  await page.reload();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();
  const footerLinks = await page.locator('footer a').evaluateAll((links) =>
    links.map((link) => ({ width: link.getBoundingClientRect().width, height: link.getBoundingClientRect().height }))
  );
  expect(footerLinks.every(({ width, height }) => width >= 44 && height >= 44)).toBe(true);
  await page.context().setOffline(true);
  await page.evaluate(() => window.dispatchEvent(new Event('offline')));
  await expect(page.getByText(/You’re offline/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Advance due reminders' })).toBeDisabled();
});

test('managed clinic entry works at 390px, 200% text, reduced motion, and keyboard only', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/start');
  await expect(page.getByRole('button', { name: 'Sign in with Microsoft' })).toBeEnabled();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();
  await page.evaluate(() => { document.documentElement.style.fontSize = '200%'; });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  const targets = await page.locator('button, a.button').evaluateAll((items) => items.map((item) => ({ width: item.getBoundingClientRect().width, height: item.getBoundingClientRect().height })));
  expect(targets.every(({ width, height }) => width >= 44 && height >= 44)).toBe(true);
});

test('unknown browser routes return an HTTP 404 with the styled recovery page', async ({ page }) => {
  const response = await page.goto('/missing-route');
  expect(response?.status()).toBe(404);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('This page has no ledger entry');
});
