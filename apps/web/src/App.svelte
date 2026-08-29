<script lang="ts">
  import { onMount, tick } from 'svelte';
  import type { AccountInfo, PublicClientApplication } from '@azure/msal-browser';
  import PulseLedger from './lib/art/PulseLedger.svelte';

  type Evidence = {
    at: string;
    kind: string;
    label: string;
    channel: string | null;
    provider_result: string | null;
    outcome: string;
    simulated: boolean;
  };
  type DemoException = {
    id: string;
    reason: string;
    next_action: string;
    owner: string | null;
    state: string;
    resolution: string | null;
    undo_available: boolean;
  };
  type Reminder = {
    id: string;
    patient_alias: string;
    appointment_time: string;
    appointment: string;
    state: string;
    due: boolean;
    events: Evidence[];
    exception: DemoException | null;
  };
  type DemoData = {
    workspace_id: string;
    clinic: { name: string; timezone: string; simulated: boolean };
    staff: { id: string; name: string }[];
    reminders: Reminder[];
  };
  type ClinicException = { id: string; reason: string; owner: string | null; state: string; resolution: string | null };
  type ClinicReminder = {
    id: string;
    source_id: string;
    patient_alias: string;
    appointment_time: string;
    status: string;
    channels: { channel: string; consent: string; consent_source: string; consent_captured_at: string }[];
    timeline: { at: number; kind: string; channel: string | null; outcome: string; provider_reference: string | null }[];
    exception: ClinicException | null;
  };
  type ClinicWorkspace = { organization_id: string; clinic_name: string; location_name: string; timezone: string; connector: { id: string; kind: string; last_received_at: number | null } | null; providers: { id: string; channel: string; kind: string; from: string; approved_template_id: string }[]; reminders: ClinicReminder[]; subscription: { tier: string | null; status: string | null } };
  type ThemeChoice = 'system' | 'light' | 'dark';

  let pagePath = typeof window === 'undefined' ? '/' : window.location.pathname;
  let demo: DemoData | null = null;
  let loading = false;
  let busy = false;
  let offline = typeof navigator === 'undefined' ? false : !navigator.onLine;
  let error = '';
  let notice = '';
  let announced = '';
  let clinic: ClinicWorkspace | null = null;
  let account: AccountInfo | null = null;
  let authClient: PublicClientApplication | null = null;
  let authReady = false;
  let connectorSecret = '';
  let pendingLicense = '';
  let themeChoice: ThemeChoice = 'system';

  const description =
    'Track appointment reminder attempts, delivery evidence, safe fallbacks, and staff-owned exceptions without replacing your clinic calendar.';
  const origin = 'https://clinic-reminder-proof.sociobot.in';

  function pageMeta(path: string) {
    if (path === '/') return { title: 'Reminder Proof — See every reminder outcome', heading: 'See every reminder outcome' };
    if (path === '/demo') return { title: 'Demo — Reminder Proof', heading: 'Today’s sample reminders' };
    if (path.startsWith('/demo/reminders/')) return { title: 'Reminder evidence — Reminder Proof', heading: 'Reminder evidence' };
    if (path === '/privacy') return { title: 'Privacy — Reminder Proof', heading: 'How Reminder Proof handles data' };
    if (path === '/terms') return { title: 'Terms — Reminder Proof', heading: 'Terms for Reminder Proof' };
    if (path === '/start') return { title: 'Start a clinic — Reminder Proof', heading: 'Connect your clinic reminders' };
    if (path === '/app' || path === '/auth/callback') return { title: 'Clinic ledger — Reminder Proof', heading: 'Clinic reminder ledger' };
    return { title: 'Page not found — Reminder Proof', heading: 'This page has no ledger entry' };
  }

  $: meta = pageMeta(pagePath);
  $: dueCount = demo?.reminders.filter((reminder) => reminder.due).length ?? 0;
  $: deliveredCount = demo?.reminders.filter((reminder) => reminder.state === 'delivered').length ?? 0;
  $: exceptionCount = demo?.reminders.filter((reminder) => reminder.exception && reminder.exception.state !== 'resolved').length ?? 0;
  $: selectedReminder = pagePath.startsWith('/demo/reminders/')
    ? demo?.reminders.find((reminder) => reminder.id === pagePath.split('/').at(-1)) ?? null
    : null;

  onMount(() => {
    const savedTheme = localStorage.getItem('reminder-proof:theme');
    if (savedTheme === 'light' || savedTheme === 'dark') themeChoice = savedTheme;
    applyTheme(themeChoice);
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');
    const syncSystemTheme = () => {
      if (themeChoice === 'system') applyTheme('system');
    };
    systemTheme.addEventListener('change', syncSystemTheme);
    const currentUrl = new URL(window.location.href);
    const returnedLicense = currentUrl.searchParams.get('license');
    if (returnedLicense) {
      sessionStorage.setItem('billing:return:clinic-reminder-proof', returnedLicense);
      currentUrl.searchParams.delete('license');
      window.history.replaceState({}, '', `${currentUrl.pathname}${currentUrl.search}${currentUrl.hash}`);
      pendingLicense = returnedLicense;
    } else {
      pendingLicense = sessionStorage.getItem('billing:return:clinic-reminder-proof') ?? '';
    }
    if (window.location.pathname === '/' && new URLSearchParams(window.location.search).get('demo') === '1') {
      navigate('/demo', true);
    } else if (isDemoPath()) {
      void loadDemo();
    }
    if (['/start', '/app', '/auth/callback'].includes(window.location.pathname)) void initializeAuth();
    const pop = () => {
      pagePath = window.location.pathname;
      error = '';
      if (isDemoPath()) void loadDemo();
      if (['/start', '/app', '/auth/callback'].includes(pagePath)) void initializeAuth();
      focusRoute();
    };
    const online = () => (offline = false);
    const offlineEvent = () => (offline = true);
    window.addEventListener('popstate', pop);
    window.addEventListener('online', online);
    window.addEventListener('offline', offlineEvent);
    return () => {
      window.removeEventListener('popstate', pop);
      window.removeEventListener('online', online);
      window.removeEventListener('offline', offlineEvent);
      systemTheme.removeEventListener('change', syncSystemTheme);
    };
  });

  function applyTheme(choice: ThemeChoice) {
    if (choice === 'system') delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = choice;
    const isDark = choice === 'dark' || (choice === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    document.querySelector<HTMLMetaElement>('meta[name="theme-color"]')?.setAttribute('content', isDark ? '#071519' : '#f3f7f5');
  }

  function setTheme(choice: ThemeChoice) {
    themeChoice = choice;
    if (choice === 'system') localStorage.removeItem('reminder-proof:theme');
    else localStorage.setItem('reminder-proof:theme', choice);
    applyTheme(choice);
  }

  function isDemoPath() {
    return pagePath === '/demo' || pagePath.startsWith('/demo/reminders/');
  }

  async function focusRoute() {
    await tick();
    const heading = document.getElementById('page-title');
    heading?.focus();
    announced = meta.heading;
  }

  function navigate(path: string, replace = false) {
    if (window.location.pathname !== path) {
      window.history[replace ? 'replaceState' : 'pushState']({}, '', path);
    }
    pagePath = path;
    error = '';
    notice = '';
    if (isDemoPath()) void loadDemo();
    void focusRoute();
  }

  function follow(event: MouseEvent, path: string) {
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
    event.preventDefault();
    navigate(path);
  }

  async function skipToMain(event: MouseEvent) {
    event.preventDefault();
    await tick();
    document.getElementById('main')?.focus();
  }

  async function request<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
      credentials: 'same-origin',
      headers: init?.body ? { 'content-type': 'application/json', ...init.headers } : init?.headers,
      ...init
    });
    if (!response.ok) {
      const detail = (await response.json().catch(() => null)) as { message?: string } | null;
      const failure = new Error(detail?.message ?? 'The sample clinic could not complete that action.') as Error & { status?: number };
      failure.status = response.status;
      throw failure;
    }
    if (response.status === 204) return undefined as T;
    return response.json() as Promise<T>;
  }

  async function loadDemo() {
    if (offline) return;
    loading = true;
    try {
      if (hasDemoSession()) {
        const state = await request<{ demo: DemoData }>('/api/v1/demo/state');
        demo = state.demo;
        rememberDemo(state.demo);
      } else {
        const created = await request<{ demo: DemoData }>('/api/v1/demo/workspaces', { method: 'POST' });
        demo = created.demo;
        rememberDemo(created.demo);
      }
    } catch (cause) {
      const failure = cause as Error & { status?: number };
      if (failure.status === 401) {
        try {
          const created = await request<{ demo: DemoData }>('/api/v1/demo/workspaces', { method: 'POST' });
          demo = created.demo;
          rememberDemo(created.demo);
        } catch (createFailure) {
          error = (createFailure as Error).message;
        }
      } else {
        error = failure.message;
      }
    } finally {
      loading = false;
    }
  }

  async function updateDemo(url: string, init?: RequestInit, success?: string, focusAfter?: string) {
    if (offline) {
      error = 'You’re offline. Sending and resolving are unavailable.';
      return;
    }
    busy = true;
    error = '';
    try {
      const response = await request<{ demo: DemoData }>(url, init);
      demo = response.demo;
      rememberDemo(response.demo);
      notice = success ?? 'Sample evidence updated.';
    } catch (cause) {
      error = (cause as Error).message;
    } finally {
      busy = false;
      if (focusAfter) {
        await tick();
        document.querySelector<HTMLElement>(focusAfter)?.focus();
      }
    }
  }

  async function resetDemo() {
    await updateDemo('/api/v1/demo/workspaces', { method: 'DELETE' }, 'The original sample clinic is restored.');
    await focusRoute();
  }

  function demoSessionKeys() {
    return Object.keys(sessionStorage).filter((key) => key.startsWith('demo:clinic-reminder-proof:'));
  }

  function hasDemoSession() {
    return typeof sessionStorage !== 'undefined' && demoSessionKeys().length > 0;
  }

  function rememberDemo(data: DemoData) {
    for (const key of demoSessionKeys()) sessionStorage.removeItem(key);
    sessionStorage.setItem(`demo:clinic-reminder-proof:${data.workspace_id}:active`, '1');
  }

  function startForReal() {
    navigate('/start');
  }

  function statusLabel(state: string) {
    return state === 'scheduled' ? 'Scheduled' : state === 'exception' ? 'Needs owner' : state[0].toUpperCase() + state.slice(1);
  }

  function marker(state: string) {
    if (state === 'delivered') return '✓';
    if (state === 'exception') return '◆';
    if (state === 'cancelled') return '■';
    return '•';
  }

  async function initializeAuth() {
    if (authClient) return;
    try {
      const config = await request<{ client_id: string; authority: string }>('/api/v1/auth/config');
      const { PublicClientApplication } = await import('@azure/msal-browser');
      authClient = new PublicClientApplication({ auth: { clientId: config.client_id, authority: config.authority, redirectUri: `${origin}/auth/callback`, postLogoutRedirectUri: `${origin}/start` }, cache: { cacheLocation: 'sessionStorage' } });
      await authClient.initialize();
      const result = await authClient.handleRedirectPromise();
      account = result?.account ?? authClient.getAllAccounts()[0] ?? null;
      authReady = true;
      if (pagePath === '/auth/callback') navigate('/app', true);
      if (account) {
        await loadClinic();
        await redeemBillingReturn();
      }
    } catch (cause) {
      error = (cause as Error).message || 'Sign-in could not start. Try again.';
      authReady = true;
    }
  }

  async function signIn() {
    if (!authClient) await initializeAuth();
    await authClient?.loginRedirect({ scopes: ['openid', 'profile', 'email'], prompt: 'select_account' });
  }

  async function signOut() {
    await authClient?.logoutRedirect({ account: account ?? undefined });
  }

  async function clinicRequest<T>(url: string, init?: RequestInit): Promise<T> {
    if (!authClient || !account) throw new Error('Sign in before opening clinic data.');
    const token = await authClient.acquireTokenSilent({ account, scopes: ['openid', 'profile', 'email'] });
    return request<T>(url, { ...init, headers: { authorization: `Bearer ${token.idToken}`, ...(init?.body ? { 'content-type': 'application/json' } : {}), ...init?.headers } });
  }

  async function redeemBillingReturn() {
    if (!pendingLicense || !clinic) return;
    try {
      clinic = await clinicRequest<ClinicWorkspace>('/api/v1/billing/return', {
        method: 'POST',
        body: JSON.stringify({ license: pendingLicense })
      });
      pendingLicense = '';
      sessionStorage.removeItem('billing:return:clinic-reminder-proof');
      notice = 'Your Sociobot Clinic plan is active.';
    } catch (cause) {
      error = (cause as Error).message;
    }
  }

  async function loadClinic() {
    loading = true;
    error = '';
    try { clinic = await clinicRequest<ClinicWorkspace>('/api/v1/clinic'); }
    catch (cause) { const failure = cause as Error & { status?: number }; if (failure.status !== 404) error = failure.message; }
    finally { loading = false; }
  }

  async function saveClinic(event: SubmitEvent) {
    event.preventDefault(); busy = true; error = '';
    const fields = new FormData(event.currentTarget as HTMLFormElement);
    try { clinic = await clinicRequest<ClinicWorkspace>('/api/v1/clinic', { method: 'POST', body: JSON.stringify({ clinic_name: fields.get('clinic_name'), location_name: fields.get('location_name'), timezone: fields.get('timezone') }) }); notice = 'Clinic workspace saved in managed storage.'; }
    catch (cause) { error = (cause as Error).message; } finally { busy = false; }
  }

  async function saveConnector(event: SubmitEvent) {
    event.preventDefault(); busy = true; error = '';
    const fields = new FormData(event.currentTarget as HTMLFormElement);
    try { const created = await clinicRequest<{ signing_secret: string }>('/api/v1/clinic/connectors', { method: 'POST', body: JSON.stringify({ kind: 'signed-calendar-webhook', webhook_secret: fields.get('webhook_secret') }) }); connectorSecret = created.signing_secret; await loadClinic(); notice = 'Signed calendar connector is ready.'; }
    catch (cause) { error = (cause as Error).message; } finally { busy = false; }
  }

  async function saveProvider(event: SubmitEvent) {
    event.preventDefault(); busy = true; error = '';
    const fields = new FormData(event.currentTarget as HTMLFormElement);
    const channel = String(fields.get('channel'));
    try { clinic = await clinicRequest<ClinicWorkspace>('/api/v1/clinic/providers', { method: 'POST', body: JSON.stringify({ channel, kind: channel === 'email' ? 'resend' : 'twilio', account_id: fields.get('account_id'), secret: fields.get('secret'), from: fields.get('from'), approved_template_id: fields.get('approved_template_id'), webhook_secret: fields.get('webhook_secret') }) }); notice = `${channel} provider saved. Credentials are encrypted.`; (event.currentTarget as HTMLFormElement).reset(); }
    catch (cause) { error = (cause as Error).message; } finally { busy = false; }
  }

  async function dispatchReminder(id: string) {
    busy = true; error = '';
    try { clinic = await clinicRequest<ClinicWorkspace>('/api/v1/clinic/reminders/dispatch', { method: 'POST', headers: { 'idempotency-key': crypto.randomUUID() }, body: JSON.stringify({ reminder_id: id }) }); notice = 'Dispatch evaluated consent and recorded the provider result.'; }
    catch (cause) { error = (cause as Error).message; } finally { busy = false; }
  }

  async function startCheckout() {
    busy = true;
    error = '';
    try {
      const checkout = await clinicRequest<{ checkout_url: string }>('/api/v1/billing/checkout', {
        method: 'POST',
        body: JSON.stringify({ tier: 'clinic' })
      });
      window.location.assign(checkout.checkout_url);
    } catch (cause) {
      error = (cause as Error).message;
      busy = false;
    }
  }

  async function saveException(id: string, action: 'assign' | 'resolve', value: string) {
    busy = true; error = '';
    try { clinic = await clinicRequest<ClinicWorkspace>(`/api/v1/clinic/exceptions/${id}/${action}`, { method: 'POST', body: JSON.stringify(action === 'assign' ? { owner: value } : { resolution: value }) }); notice = action === 'assign' ? 'Exception owner saved.' : 'Exception resolved.'; }
    catch (cause) { error = (cause as Error).message; } finally { busy = false; }
  }

  async function exportClinic() {
    if (!authClient || !account) return;
    const token = await authClient.acquireTokenSilent({ account, scopes: ['openid', 'profile', 'email'] });
    const response = await fetch('/api/v1/clinic/export', { headers: { authorization: `Bearer ${token.idToken}` } });
    if (!response.ok) { error = 'Clinic data could not be exported. Sign in again and retry.'; return; }
    const link = document.createElement('a'); link.href = URL.createObjectURL(await response.blob()); link.download = 'reminder-proof-export.json'; link.click(); URL.revokeObjectURL(link.href);
  }

  async function deleteClinic() {
    if (!clinic || !window.confirm(`Delete ${clinic.clinic_name} and its reminder evidence? This cannot be undone.`)) return;
    try { await clinicRequest<void>('/api/v1/clinic', { method: 'DELETE', headers: { 'x-confirm-delete': clinic.organization_id } }); clinic = null; notice = 'Clinic workspace deleted.'; }
    catch (cause) { error = (cause as Error).message; }
  }
</script>

<svelte:head>
  <title>{meta.title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={`${origin}${pagePath}`} />
  <meta property="og:title" content={meta.title} />
  <meta property="og:description" content={description} />
  <meta property="og:type" content="website" />
  <meta property="og:url" content={`${origin}${pagePath}`} />
  <meta property="og:image" content={`${origin}/social-card.svg`} />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={meta.title} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={`${origin}/social-card.svg`} />
</svelte:head>

<a class="skip-link" href="#main" onclick={skipToMain}>Skip to main content</a>
<div class="route-announcer" aria-live="polite">{announced}</div>

<header class="site-header">
  <a class="wordmark" href="/" aria-label="Reminder Proof home" onclick={(event) => follow(event, '/') }>
    <span class="wordmark-mark" aria-hidden="true"><i></i><i></i></span>
    <span>Reminder Proof</span>
  </a>
  <div class="header-controls">
    <nav aria-label="Primary navigation">
      <a href="/demo" onclick={(event) => follow(event, '/demo')}>Demo</a>
      <a href="/start" onclick={(event) => follow(event, '/start')}>For clinics</a>
      <a href="/#how">How it works</a>
      <a href="/privacy" onclick={(event) => follow(event, '/privacy')}>Privacy</a>
    </nav>
    <label class="theme-control">Theme
      <select aria-label="Color theme" value={themeChoice} onchange={(event) => setTheme(event.currentTarget.value as ThemeChoice)}>
        <option value="system">System</option>
        <option value="light">Clinic daylight</option>
        <option value="dark">After hours</option>
      </select>
    </label>
  </div>
</header>

<main id="main" tabindex="-1">
  {#if pagePath === '/'}
    <section class="landing-hero" aria-labelledby="page-title">
      <div class="hero-copy">
        <p class="eyebrow">Reminder proof for independent clinics</p>
        <h1 id="page-title" tabindex="-1">See every reminder outcome.</h1>
        <p class="lede">For independent clinics that need delivery proof and a clear next step when reminders fail.</p>
        <div class="hero-actions">
          <a class="button primary" href="/demo" onclick={(event) => follow(event, '/demo')}>Try it with sample data</a>
          <p>Opens a sample clinic. Nothing touches real clinic data.</p>
        </div>
        <ul class="plain-facts" aria-label="Product facts">
          <li>Demo actions use sample data only.</li>
          <li>Reminder contents exclude clinical notes.</li>
        <li>Clinic costs $79 per location each month.</li>
        </ul>
      </div>
      <div class="hero-preview" aria-label="Sample delivery ledger preview">
        <PulseLedger />
        <div class="preview-note preview-delivered"><b>✓ Delivered</b><span>SMS · Simulated</span></div>
        <div class="preview-note preview-fallback"><b>→ Fallback</b><span>WhatsApp → Email</span></div>
        <div class="preview-note preview-exception"><b>◆ Needs owner</b><span>Consent blocked</span></div>
      </div>
    </section>

    <section class="work-section" aria-labelledby="follow-title">
      <div>
        <p class="eyebrow">A proof ledger, not another calendar</p>
        <h2 id="follow-title">Follow one reminder from schedule to outcome.</h2>
      </div>
      <p class="section-lede">See the source, consent check, each attempt, provider result, and staff resolution in order.</p>
    </section>

    <section id="how" class="steps" aria-labelledby="how-title">
      <h2 id="how-title">How the sample clinic works</h2>
      <ol>
        <li><span>01</span><div><h3>Check consent first</h3><p>A blocked channel becomes an exception before any simulated provider attempt.</p></div></li>
        <li><span>02</span><div><h3>Use the next allowed channel</h3><p>Reminder Proof tries a fallback only when consent and the clinic policy allow it.</p></div></li>
        <li><span>03</span><div><h3>Give every failure an owner</h3><p>Blocked and exhausted reminders stay visible until a staff member resolves them.</p></div></li>
      </ol>
    </section>

    <section class="boundary-section" aria-labelledby="boundary-title">
      <div><p class="eyebrow">Plain boundaries</p><h2 id="boundary-title">This does not replace your calendar or EMR.</h2></div>
      <p>Reminder Proof stores no clinical notes and sends no marketing campaigns. The public demo stays separate from managed clinic data.</p>
    </section>

    <section class="pricing-section" aria-labelledby="pricing-title">
      <div><p class="eyebrow">Monthly plan</p><h2 id="pricing-title">One clear clinic price.</h2></div>
      <div class="price-line"><strong>$79</strong><span>per location each month</span></div>
      <p><a class="button secondary" href="/start" onclick={(event) => follow(event, '/start')}>Connect your clinic</a></p>
    </section>
  {:else if pagePath === '/demo'}
    <section class="app-page" aria-labelledby="page-title">
      <div class="demo-banner" role="status">
        <span><strong>Demo</strong> — sample data, nothing is saved to your clinic.</span>
        <div><button class="text-button" onclick={resetDemo} disabled={busy || loading}>Reset demo</button><button class="text-button" onclick={startForReal}>Start for real</button></div>
      </div>
      <div class="page-heading">
        <div><p class="eyebrow">{demo?.clinic.name ?? 'Loading sample clinic'}</p><h1 id="page-title" tabindex="-1">Today’s sample reminders</h1><p>Every provider result is simulated. No real reminder is sent.</p></div>
        <button class="button primary" onclick={() => updateDemo('/api/v1/demo/reminders/advance-due', { method: 'POST' }, 'Due sample reminders advanced.') } disabled={busy || loading || offline}>Advance due reminders</button>
      </div>
      {#if offline}<div class="state-notice warning" role="status">You’re offline. This ledger was last updated in this browser. Sending and resolving are unavailable.</div>{/if}
      {#if error}<div class="state-notice danger" role="alert">{error} <button class="text-button" onclick={() => void loadDemo()}>Try again</button></div>{/if}
      {#if notice}<div class="state-notice success" role="status">{notice}</div>{/if}
      {#if loading && !demo}
        <div class="state-panel" role="status">Loading the sample ledger…</div>
      {:else if demo}
        <div class="summary-grid" aria-label="Sample reminder summary">
          <div><span>Due</span><strong>{dueCount}</strong><small>sample reminders</small></div>
          <div><span>Delivered</span><strong>{deliveredCount}</strong><small>with provider evidence</small></div>
          <div><span>Exceptions</span><strong>{exceptionCount}</strong><small>need a person</small></div>
        </div>
        <section class="ledger-panel" aria-labelledby="ledger-title">
          <div class="panel-heading"><div><p class="eyebrow">Delivery ledger</p><h2 id="ledger-title">Evidence for each sample appointment</h2></div><span class="simulated-label">Simulated provider events</span></div>
          <ul class="ledger-list">
            {#each demo.reminders as reminder}
              <li class:needs-owner={reminder.state === 'exception'}>
                <span class={`status-mark ${reminder.state}`} aria-hidden="true">{marker(reminder.state)}</span>
                <div class="appointment"><strong>{reminder.appointment_time} · {reminder.patient_alias}</strong><span>{reminder.appointment}</span></div>
                <div class="row-outcome"><span class={`status-word ${reminder.state}`}>{statusLabel(reminder.state)}</span><span>{reminder.events.at(-1)?.outcome}</span></div>
                <a href={`/demo/reminders/${reminder.id}`} onclick={(event) => follow(event, `/demo/reminders/${reminder.id}`)}>View evidence<span class="sr-only"> for {reminder.patient_alias}</span></a>
              </li>
            {/each}
          </ul>
        </section>
        <section class="exception-panel" aria-labelledby="exceptions-title">
          <div class="panel-heading"><div><p class="eyebrow">Exception queue</p><h2 id="exceptions-title">Reminders that need a person</h2></div><span>{exceptionCount} open</span></div>
          {#each demo.reminders.filter((reminder) => reminder.exception) as reminder}
            {@const task = reminder.exception!}
            <article class="exception-row">
              <div><h3>{reminder.appointment_time} · {reminder.patient_alias}</h3><p>{task.reason}</p><p class="next-action">{task.next_action}</p></div>
              <div class="exception-controls">
                <label>Owner<select aria-label={`Owner for ${reminder.patient_alias}`} value={task.owner ?? ''} disabled={busy || loading || offline} onchange={(event) => updateDemo(`/api/v1/demo/exceptions/${task.id}/assign`, { method: 'POST', body: JSON.stringify({ owner: (event.currentTarget as HTMLSelectElement).value }) }, 'Owner saved.') }><option value="" disabled>Choose owner</option>{#each demo.staff as staff}<option value={staff.name}>{staff.name}</option>{/each}</select></label>
                {#if task.state !== 'resolved'}
                  <button class="button secondary" disabled={!task.owner || busy || offline} onclick={() => updateDemo(`/api/v1/demo/exceptions/${task.id}/resolve`, { method: 'POST', body: JSON.stringify({ resolution: 'Called patient' }) }, 'Exception resolved as Called patient.', `#undo-${task.id}`)}>Resolve as Called patient</button>
                {:else}
                  <p class="resolution"><strong>Resolved:</strong> {task.resolution}</p>
                  <button id={`undo-${task.id}`} class="text-button" disabled={busy || offline || !task.undo_available} onclick={() => updateDemo(`/api/v1/demo/exceptions/${task.id}/undo`, { method: 'POST' }, 'Resolution undone. The exception remains assigned.')}>Undo resolution</button>
                {/if}
              </div>
            </article>
          {/each}
        </section>
      {/if}
    </section>
  {:else if pagePath.startsWith('/demo/reminders/')}
    <section class="app-page" aria-labelledby="page-title">
      <div class="demo-banner" role="status"><span><strong>Demo</strong> — sample data, nothing is saved to your clinic.</span><div><button class="text-button" onclick={resetDemo} disabled={busy || loading}>Reset demo</button><button class="text-button" onclick={startForReal}>Start for real</button></div></div>
      {#if loading && !demo}<div class="state-panel" role="status">Loading sample evidence…</div>
      {:else if !selectedReminder}<div class="state-panel"><h1 id="page-title" tabindex="-1">This sample reminder has no ledger entry</h1><p>Choose a reminder from the sample ledger.</p><a class="button primary" href="/demo" onclick={(event) => follow(event, '/demo')}>Return to sample ledger</a></div>
      {:else}
        <a class="back-link" href="/demo" onclick={(event) => follow(event, '/demo')}>← Back to sample ledger</a>
        <div class="detail-heading"><div><p class="eyebrow">Simulated evidence</p><h1 id="page-title" tabindex="-1">Evidence for {selectedReminder.appointment_time} appointment</h1><p>{selectedReminder.patient_alias} · {selectedReminder.appointment}</p></div><span class={`status-word ${selectedReminder.state}`}>{statusLabel(selectedReminder.state)}</span></div>
        {#if error}<div class="state-notice danger" role="alert">{error}</div>{/if}
        <section class="timeline-panel" aria-labelledby="timeline-title"><h2 id="timeline-title">Timeline</h2><ol class="timeline">
          {#each selectedReminder.events as event}
            <li><span class={`timeline-marker ${event.kind}`} aria-hidden="true">{event.kind === 'attempt' ? '→' : event.kind === 'consent' ? '□' : event.kind === 'response' ? '↳' : '•'}</span><div><p><time>{event.at}</time> · {event.label}</p><dl><div><dt>Outcome</dt><dd>{event.outcome}</dd></div>{#if event.channel}<div><dt>Channel</dt><dd>{event.channel}</dd></div>{/if}{#if event.provider_result}<div><dt>Provider result</dt><dd>{event.provider_result}</dd></div>{/if}<div><dt>Provider mode</dt><dd>Simulated</dd></div></dl></div></li>
          {/each}
        </ol></section>
        {#if selectedReminder.state === 'scheduled'}<button class="button primary" disabled={busy || offline} onclick={() => updateDemo(`/api/v1/demo/reminders/${selectedReminder?.id}/advance`, { method: 'POST' }, 'Sample reminder advanced.')}>Advance this sample reminder</button>{/if}
        {#if selectedReminder.exception}<section class="detail-exception" aria-labelledby="detail-exception-title"><h2 id="detail-exception-title">Exception</h2><p>{selectedReminder.exception.reason}</p><p>{selectedReminder.exception.next_action}</p><a href="/demo" onclick={(event) => follow(event, '/demo')}>Manage this exception in the sample queue</a></section>{/if}
      {/if}
    </section>
  {:else if pagePath === '/start'}
    <section class="app-page real-page" aria-labelledby="page-title">
      <div class="page-heading"><div><p class="eyebrow">Managed clinic workflow</p><h1 id="page-title" tabindex="-1">Connect your clinic reminders</h1><p>Sign in, connect your calendar, and send approved reminders with delivery proof and staff-owned exceptions.</p></div></div>
      <div class="workflow-grid" aria-label="Clinic workflow">
        <div><span>01</span><h2>Sign in safely</h2><p>Sociobot Microsoft Entra protects each clinic workspace.</p></div>
        <div><span>02</span><h2>Connect approved services</h2><p>A signed calendar feed and encrypted provider credentials keep systems separate.</p></div>
        <div><span>03</span><h2>Follow every outcome</h2><p>Consent, fallback attempts, signed receipts, and staff action stay in one timeline.</p></div>
      </div>
      <div class="state-notice warning"><strong>Use minimum data:</strong> send patient aliases, contact destinations, consent evidence, and appointment times. Do not send clinical notes.</div>
      {#if error}<div class="state-notice danger" role="alert">{error}</div>{/if}
      <div class="start-actions">
        <button class="button primary" onclick={signIn} disabled={!authReady || busy}>{authReady ? 'Sign in with Microsoft' : 'Preparing secure sign-in…'}</button>
        <a class="button secondary" href="/demo" onclick={(event) => follow(event, '/demo')}>Try sample data first</a>
        <span>Subscription checkout opens after you sign in and create a clinic workspace.</span>
        <span>$79 per location each month. Delivery-provider fees are separate.</span>
      </div>
    </section>
  {:else if pagePath === '/app' || pagePath === '/auth/callback'}
    <section class="app-page real-page" aria-labelledby="page-title">
      <div class="page-heading"><div><p class="eyebrow">Managed clinic workspace</p><h1 id="page-title" tabindex="-1">Clinic reminder ledger</h1><p>Calendar intake, approved dispatch, receipts, and exceptions are stored for your signed-in clinic.</p></div>{#if account}<button class="text-button" onclick={signOut}>Sign out</button>{/if}</div>
      {#if error}<div class="state-notice danger" role="alert">{error}</div>{/if}
      {#if notice}<div class="state-notice success" role="status">{notice}</div>{/if}
      {#if !authReady || loading}<div class="state-panel" role="status">Loading your secure clinic workspace…</div>
      {:else if !account}<div class="state-panel"><h2>Sign in to continue</h2><p>Clinic data is never available through the public demo.</p><button class="button primary" onclick={signIn}>Sign in with Microsoft</button></div>
      {:else if !clinic}
        <form class="setup-form" onsubmit={saveClinic}>
          <div><h2>Create your clinic workspace</h2><p>This managed workspace belongs to your Entra identity.</p></div>
          <label>Clinic name<input name="clinic_name" required maxlength="100" autocomplete="organization" /></label>
          <label>Location name<input name="location_name" required maxlength="100" /></label>
          <label>Timezone<input name="timezone" required maxlength="64" value="Europe/London" aria-describedby="timezone-help" /><small id="timezone-help">Use an IANA timezone such as Europe/London.</small></label>
          <button class="button primary" disabled={busy}>Create clinic workspace</button>
        </form>
      {:else}
        <div class="summary-grid" aria-label="Clinic reminder summary"><div><span>Reminders</span><strong>{clinic.reminders.length}</strong><small>from the connector</small></div><div><span>Provider proof</span><strong>{clinic.reminders.filter((item) => ['delivered', 'read'].includes(item.status)).length}</strong><small>terminal receipts</small></div><div><span>Exceptions</span><strong>{clinic.reminders.filter((item) => item.exception && item.exception.state !== 'resolved').length}</strong><small>need a person</small></div></div>
        <section class="setup-columns" aria-label="Clinic connection setup">
          <form class="setup-form" onsubmit={saveConnector}><div><p class="eyebrow">Source</p><h2>Signed calendar connector</h2><p>Your EMR or calendar posts appointments through a signed HTTPS request.</p></div><label>Signing secret<input name="webhook_secret" type="password" minlength="16" maxlength="200" required autocomplete="new-password" /></label><button class="button secondary" disabled={busy}>Create connector</button>{#if clinic.connector}<p class="config-proof"><strong>Connected:</strong> {clinic.connector.id}</p>{/if}{#if connectorSecret}<p class="state-notice warning"><strong>Copy now:</strong> {connectorSecret}</p>{/if}</form>
          <form class="setup-form" onsubmit={saveProvider}><div><p class="eyebrow">Delivery</p><h2>Approved provider</h2><p>Twilio sends SMS or approved WhatsApp. Resend sends email.</p></div><label>Channel<select name="channel" required><option value="sms">SMS</option><option value="email">Email</option><option value="whatsapp">WhatsApp</option></select></label><label>Account ID<input name="account_id" maxlength="200" /></label><label>Provider credential<input name="secret" type="password" required maxlength="300" autocomplete="new-password" /></label><label>Approved sender<input name="from" required maxlength="300" /></label><label>Approved template ID<input name="approved_template_id" required maxlength="300" /></label><label>Receipt signing secret<input name="webhook_secret" type="password" required minlength="16" maxlength="300" autocomplete="new-password" /><small>For Resend, enter its webhook secret (starts with <code>whsec_</code>). Twilio verifies callbacks with the provider credential.</small></label><button class="button secondary" disabled={busy}>Save provider</button>{#if clinic.providers.length > 0}<ul class="config-list">{#each clinic.providers as provider}<li><strong>{provider.channel} · {provider.kind}</strong><span>Receipt URL: {origin}/api/v1/providers/{provider.kind === 'twilio' ? 'twilio/' : 'resend/'}{provider.id}/receipts</span></li>{/each}</ul>{/if}</form>
        </section>
        <section class="ledger-panel" aria-labelledby="clinic-ledger-title"><div class="panel-heading"><div><p class="eyebrow">Delivery ledger</p><h2 id="clinic-ledger-title">Real reminder evidence</h2></div><span>{clinic.location_name} · {clinic.timezone}</span></div>
          {#if clinic.reminders.length === 0}<div class="state-panel"><h3>No appointments received</h3><p>Connect your source and send its first signed appointment batch.</p></div>{:else}<ul class="real-list">{#each clinic.reminders as reminder}<li><div><strong>{reminder.appointment_time} · {reminder.patient_alias}</strong><span>{reminder.source_id}</span></div><div><strong>{statusLabel(reminder.status)}</strong><span>{reminder.timeline.at(-1)?.outcome}</span></div><div>{reminder.channels.map((item) => `${item.channel}: ${item.consent}`).join(' → ')}</div><button class="button secondary" onclick={() => dispatchReminder(reminder.id)} disabled={busy || reminder.status !== 'scheduled'}>Dispatch approved reminder</button>{#if reminder.exception}<label>Exception owner<input value={reminder.exception.owner ?? ''} aria-label={`Exception owner for ${reminder.patient_alias}`} onblur={(event) => saveException(reminder.exception!.id, 'assign', event.currentTarget.value)} /></label>{#if reminder.exception.owner && reminder.exception.state !== 'resolved'}<button class="text-button" onclick={() => saveException(reminder.exception!.id, 'resolve', 'Called patient')}>Resolve as Called patient</button>{/if}{/if}</li>{/each}</ul>{/if}
        </section>
        <section class="billing-panel" aria-labelledby="billing-title"><div><p class="eyebrow">Subscription</p><h2 id="billing-title">Clinic plan</h2><p>$79 per location each month. Delivery-provider fees are separate. Checkout and subscription status are handled by Sociobot.</p><p><strong>Status:</strong> {clinic.subscription.status === 'active' ? 'Active' : 'Required before live dispatch'}</p></div><button class="button primary" onclick={startCheckout} disabled={busy}>Subscribe through Sociobot <span class="sr-only">(opens Sociobot checkout)</span></button></section>
        <div class="data-actions"><button class="text-button" onclick={exportClinic}>Export clinic data</button><button class="text-button danger-action" onclick={deleteClinic}>Delete clinic workspace</button><span>Export includes minimized reminder evidence and exceptions.</span></div>
      {/if}
    </section>
  {:else if pagePath === '/privacy'}
    <section class="legal-page" aria-labelledby="page-title"><p class="eyebrow">Privacy</p><h1 id="page-title" tabindex="-1">How Reminder Proof handles data</h1><p class="lede">The public demo and signed-in clinic workspaces are separate.</p><h2>Public demo</h2><p>An HttpOnly, Secure browser cookie holds compact fictional sample state. It expires within 24 hours.</p><h2>Managed clinic data</h2><p>Clinic data is stored on the service and scoped to the stable Entra account ID. Provider credentials are encrypted and never returned to the browser.</p><h2>Minimum patient data</h2><p>Use aliases, contact destinations, consent evidence, and appointment times. Do not store clinical notes, diagnoses, or treatment details.</p><h2>Providers and billing</h2><p>Real dispatch sends the approved reminder to the configured Twilio or Resend endpoint. Subscription checkout and verification use Sociobot.</p><h2>Tracking and control</h2><p>No tracking script loads. Signed-in clinics can export or delete their workspace. Do not use Reminder Proof as a medical record.</p></section>
  {:else if pagePath === '/terms'}
    <section class="legal-page" aria-labelledby="page-title"><p class="eyebrow">Terms</p><h1 id="page-title" tabindex="-1">Terms for Reminder Proof</h1><p class="lede">Reminder Proof is a delivery-evidence layer for clinic reminder operations. It is not a medical system and does not provide medical advice.</p><h2>Sample clinic</h2><p>The public demo uses fictional sample data and simulated provider outcomes. Do not enter real patient or clinic information into it.</p><h2>Monthly plan</h2><p>The Clinic plan is $79 per location each month. Delivery-provider fees are separate. Sociobot is the checkout and subscription service.</p><h2>Clinic responsibilities</h2><p>Clinic operators remain responsible for consent, lawful messaging, approved provider accounts, source records, and local privacy obligations.</p></section>
  {:else}
    <section class="not-found" aria-labelledby="page-title"><PulseLedger label="A quiet empty proof ledger." /><div><p class="eyebrow">404</p><h1 id="page-title" tabindex="-1">This page has no ledger entry</h1><p>Return to the page that explains the sample clinic.</p><a class="button primary" href="/" onclick={(event) => follow(event, '/')}>Go to Reminder Proof</a></div></section>
  {/if}
</main>

<footer class="site-footer"><p>Reminder Proof records delivery evidence and staff-owned exceptions around an existing clinic calendar.</p><nav aria-label="Footer"><a href="/privacy" onclick={(event) => follow(event, '/privacy')}>Privacy</a><a href="/terms" onclick={(event) => follow(event, '/terms')}>Terms</a><a href="https://sociobot.in" rel="external">Built by Param Factory <span class="sr-only">(opens Sociobot)</span></a></nav><small>managed-clinic-workflow</small></footer>
