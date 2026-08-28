<script lang="ts">
  import { onMount, tick } from 'svelte';
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
  type RealReminder = {
    id: string;
    patient_alias: string;
    appointment_time: string;
    primary_channel: string;
    consent: string;
    primary_result: string;
    fallback_channel: string;
    fallback_consent: string;
    fallback_result: string;
    owner: string;
  };

  let pagePath = typeof window === 'undefined' ? '/' : window.location.pathname;
  let demo: DemoData | null = null;
  let loading = false;
  let busy = false;
  let offline = typeof navigator === 'undefined' ? false : !navigator.onLine;
  let error = '';
  let notice = '';
  let announced = '';
  let realReminders: RealReminder[] = [];
  let importError = '';
  const realStorageKey = 'real:clinic-reminder-proof:ledger';

  const description =
    'Track appointment reminder attempts, delivery evidence, safe fallbacks, and staff-owned exceptions without replacing your clinic calendar.';
  const origin = 'https://clinic-reminder-proof.sociobot.in';

  function pageMeta(path: string) {
    if (path === '/') return { title: 'Reminder Proof — See every reminder outcome', heading: 'See every reminder outcome' };
    if (path === '/demo') return { title: 'Demo — Reminder Proof', heading: 'Today’s sample reminders' };
    if (path.startsWith('/demo/reminders/')) return { title: 'Reminder evidence — Reminder Proof', heading: 'Reminder evidence' };
    if (path === '/privacy') return { title: 'Privacy — Reminder Proof', heading: 'How Reminder Proof handles data' };
    if (path === '/terms') return { title: 'Terms — Reminder Proof', heading: 'Terms for Reminder Proof' };
    if (path === '/start') return { title: 'Import reminder evidence — Reminder Proof', heading: 'Audit real reminder results' };
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
    try {
      realReminders = JSON.parse(localStorage.getItem(realStorageKey) ?? '[]') as RealReminder[];
    } catch {
      localStorage.removeItem(realStorageKey);
    }
    if (window.location.pathname === '/' && new URLSearchParams(window.location.search).get('demo') === '1') {
      navigate('/demo', true);
    } else if (isDemoPath()) {
      void loadDemo();
    }
    const pop = () => {
      pagePath = window.location.pathname;
      error = '';
      if (isDemoPath()) void loadDemo();
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
    };
  });

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

  async function updateDemo(url: string, init?: RequestInit, success?: string) {
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

  function parseCsv(text: string): string[][] {
    const rows: string[][] = [];
    let row: string[] = [];
    let field = '';
    let quoted = false;
    for (let index = 0; index < text.length; index += 1) {
      const character = text[index];
      if (character === '"' && quoted && text[index + 1] === '"') {
        field += '"';
        index += 1;
      } else if (character === '"') quoted = !quoted;
      else if (character === ',' && !quoted) {
        row.push(field.trim());
        field = '';
      } else if ((character === '\n' || character === '\r') && !quoted) {
        if (character === '\r' && text[index + 1] === '\n') index += 1;
        row.push(field.trim());
        if (row.some(Boolean)) rows.push(row);
        row = [];
        field = '';
      } else field += character;
    }
    row.push(field.trim());
    if (row.some(Boolean)) rows.push(row);
    return rows;
  }

  async function importEvidence(event: Event) {
    importError = '';
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    if (file.size > 1_000_000) {
      importError = 'That file is over 1 MB. Export a smaller date range and try again.';
      input.value = '';
      return;
    }
    const rows = parseCsv(await file.text());
    const headers = rows.shift()?.map((header) => header.toLowerCase()) ?? [];
    const required = ['reminder_id', 'patient_alias', 'appointment_time', 'primary_channel', 'consent', 'primary_result'];
    const missing = required.filter((header) => !headers.includes(header));
    if (missing.length > 0) {
      importError = `The CSV is missing: ${missing.join(', ')}.`;
      input.value = '';
      return;
    }
    const value = (row: string[], name: string) => row[headers.indexOf(name)]?.trim() ?? '';
    const imported = rows.map((row) => ({
      id: value(row, 'reminder_id'),
      patient_alias: value(row, 'patient_alias'),
      appointment_time: value(row, 'appointment_time'),
      primary_channel: value(row, 'primary_channel'),
      consent: value(row, 'consent').toLowerCase(),
      primary_result: value(row, 'primary_result').toLowerCase(),
      fallback_channel: value(row, 'fallback_channel'),
      fallback_consent: value(row, 'fallback_consent').toLowerCase(),
      fallback_result: value(row, 'fallback_result').toLowerCase(),
      owner: ''
    })).filter((item) => item.id && item.patient_alias && item.appointment_time);
    if (imported.length === 0) {
      importError = 'No complete reminder rows were found. Check the template and try again.';
      input.value = '';
      return;
    }
    realReminders = imported;
    localStorage.setItem(realStorageKey, JSON.stringify(realReminders));
    notice = `${imported.length} reminder results imported.`;
    input.value = '';
  }

  function realOutcome(item: RealReminder) {
    if (item.consent !== 'allowed') return 'Blocked before dispatch';
    if (['delivered', 'replied'].includes(item.primary_result)) return 'Delivered';
    if (item.fallback_channel && item.fallback_consent === 'allowed' && ['delivered', 'replied'].includes(item.fallback_result)) return 'Delivered by fallback';
    if (['queued', 'accepted', 'pending'].includes(item.primary_result)) return 'Awaiting delivery proof';
    return 'Needs staff action';
  }

  function setRealOwner(index: number, owner: string) {
    realReminders[index].owner = owner.trim();
    realReminders = [...realReminders];
    localStorage.setItem(realStorageKey, JSON.stringify(realReminders));
  }

  function exportRealLedger() {
    const escaped = (value: string) => `"${value.replaceAll('"', '""')}"`;
    const header = 'reminder_id,patient_alias,appointment_time,outcome,owner';
    const rows = realReminders.map((item) => [item.id, item.patient_alias, item.appointment_time, realOutcome(item), item.owner].map(escaped).join(','));
    const link = document.createElement('a');
    link.href = URL.createObjectURL(new Blob([[header, ...rows].join('\n')], { type: 'text/csv' }));
    link.download = 'reminder-proof-ledger.csv';
    link.click();
    URL.revokeObjectURL(link.href);
  }

  function clearRealLedger() {
    if (!window.confirm('Delete the imported ledger from this browser?')) return;
    realReminders = [];
    localStorage.removeItem(realStorageKey);
    notice = 'Imported reminder data was deleted from this browser.';
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
  <nav aria-label="Primary navigation">
    <a href="/demo" onclick={(event) => follow(event, '/demo')}>Demo</a>
    <a href="/#how">How it works</a>
    <a href="/privacy" onclick={(event) => follow(event, '/privacy')}>Privacy</a>
  </nav>
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
          <li>Clinic costs $79 per location each month, plus published messaging charges.</li>
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
      <p>Reminder Proof stores no clinical notes and sends no marketing campaigns. The M1 demo simulates provider outcomes; it sends no real messages.</p>
    </section>

    <section class="pricing-section" aria-labelledby="pricing-title">
      <div><p class="eyebrow">Monthly plan</p><h2 id="pricing-title">One clear clinic price.</h2></div>
      <div class="price-line"><strong>$79</strong><span>per location each month<br />plus published messaging charges</span></div>
      <p>Billing is not available in this milestone. Subscriptions arrive after accounts and clinic data are in place.</p>
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
                  <button class="button secondary" disabled={!task.owner || busy || offline} onclick={() => updateDemo(`/api/v1/demo/exceptions/${task.id}/resolve`, { method: 'POST', body: JSON.stringify({ resolution: 'Called patient' }) }, 'Exception resolved as Called patient.')}>Resolve as Called patient</button>
                {:else}
                  <p class="resolution"><strong>Resolved:</strong> {task.resolution}</p>
                  <button class="text-button" disabled={busy || offline || !task.undo_available} onclick={() => updateDemo(`/api/v1/demo/exceptions/${task.id}/undo`, { method: 'POST' }, 'Resolution undone. The exception remains assigned.')}>Undo resolution</button>
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
      <div class="page-heading"><div><p class="eyebrow">Real data · local browser</p><h1 id="page-title" tabindex="-1">Audit real reminder results</h1><p>Import a CSV export from your calendar or messaging provider. Reminder Proof classifies proof and exceptions without sending a message.</p></div></div>
      <div class="state-notice warning"><strong>Before you import:</strong> use patient aliases, not names. Data stays in this browser. This tool is not a medical record.</div>
      <section class="import-panel" aria-labelledby="import-title">
        <div><h2 id="import-title">Import reminder evidence</h2><p>Required columns: reminder_id, patient_alias, appointment_time, primary_channel, consent, primary_result. Optional fallback columns record the next allowed attempt.</p></div>
        <label class="file-button">Choose CSV file<input type="file" accept=".csv,text/csv" onchange={importEvidence} /></label>
      </section>
      {#if importError}<div class="state-notice danger" role="alert">{importError}</div>{/if}
      {#if notice}<div class="state-notice success" role="status">{notice}</div>{/if}
      {#if realReminders.length === 0}
        <div class="state-panel"><h2>No reminder evidence imported</h2><p>Your classified results and staff exceptions will appear here.</p></div>
      {:else}
        <div class="real-toolbar"><p><strong>{realReminders.length}</strong> imported reminders · <strong>{realReminders.filter((item) => realOutcome(item).includes('Delivered')).length}</strong> delivered · <strong>{realReminders.filter((item) => !realOutcome(item).includes('Delivered')).length}</strong> need review</p><div><button class="button secondary" onclick={exportRealLedger}>Export proof CSV</button><button class="text-button" onclick={clearRealLedger}>Delete imported data</button></div></div>
        <section class="ledger-panel" aria-labelledby="real-ledger-title">
          <div class="panel-heading"><div><p class="eyebrow">Imported provider evidence</p><h2 id="real-ledger-title">Reminder proof ledger</h2></div></div>
          <ul class="real-list">
            {#each realReminders as item, index}
              <li>
                <div><strong>{item.appointment_time} · {item.patient_alias}</strong><span>{item.id}</span></div>
                <dl><div><dt>Primary</dt><dd>{item.primary_channel} · {item.consent || 'unknown'} · {item.primary_result || 'unknown'}</dd></div>{#if item.fallback_channel}<div><dt>Fallback</dt><dd>{item.fallback_channel} · {item.fallback_consent || 'unknown'} · {item.fallback_result || 'unknown'}</dd></div>{/if}</dl>
                <strong class:real-alert={!realOutcome(item).includes('Delivered')}>{realOutcome(item)}</strong>
                {#if !realOutcome(item).includes('Delivered')}<label>Exception owner<input aria-label={`Exception owner for ${item.patient_alias}`} value={item.owner} onblur={(event) => setRealOwner(index, event.currentTarget.value)} /></label>{/if}
              </li>
            {/each}
          </ul>
        </section>
      {/if}
      <section class="boundary-section compact" aria-labelledby="real-boundary"><div><h2 id="real-boundary">What this real workflow does not do</h2></div><p>It does not dispatch patient messages, connect to an EMR, or verify provider signatures. Those steps require clinic credentials, contracts, and privacy review.</p></section>
    </section>
  {:else if pagePath === '/privacy'}
    <section class="legal-page" aria-labelledby="page-title"><p class="eyebrow">Privacy</p><h1 id="page-title" tabindex="-1">How Reminder Proof handles data</h1><p class="lede">The demo creates a random, short-lived sample workspace. It contains fictional aliases and simulated provider events.</p><h2>What the demo stores</h2><p>An HttpOnly, Secure browser cookie holds compact sample state. It expires within 24 hours. Reset demo replaces it with a new sample.</p><h2>Real CSV imports</h2><p>The real evidence tool stores imported rows only in this browser. Delete imported data removes that local copy. Use aliases and do not import clinical notes.</p><h2>What the site does not do</h2><p>It does not load a tracking script. The demo does not call a payment service, messaging provider, or clinic connector.</p><h2>Current limit</h2><p>Accounts, managed clinic storage, live provider sending, and subscriptions are not available. Do not use the service as a medical record.</p></section>
  {:else if pagePath === '/terms'}
    <section class="legal-page" aria-labelledby="page-title"><p class="eyebrow">Terms</p><h1 id="page-title" tabindex="-1">Terms for Reminder Proof</h1><p class="lede">Reminder Proof is a delivery-evidence layer for clinic reminder operations. It is not a medical system and does not provide medical advice.</p><h2>Sample clinic</h2><p>The public demo uses fictional sample data and simulated provider outcomes. Do not enter real patient or clinic information into it.</p><h2>Planned monthly plan</h2><p>The Clinic plan is $79 per location each month, plus published messaging charges. Billing is unavailable in M1. Sociobot and Dodo will be the merchant flow after accounts ship.</p><h2>Clinic responsibilities</h2><p>Clinic operators remain responsible for consent, lawful messaging, source records, and their local privacy obligations.</p></section>
  {:else}
    <section class="not-found" aria-labelledby="page-title"><PulseLedger label="A quiet empty proof ledger." /><div><p class="eyebrow">404</p><h1 id="page-title" tabindex="-1">This page has no ledger entry</h1><p>Return to the page that explains the sample clinic.</p><a class="button primary" href="/" onclick={(event) => follow(event, '/')}>Go to Reminder Proof</a></div></section>
  {/if}
</main>

<footer class="site-footer"><p>Reminder Proof records delivery evidence and staff-owned exceptions around an existing clinic calendar.</p><nav aria-label="Footer"><a href="/privacy" onclick={(event) => follow(event, '/privacy')}>Privacy</a><a href="/terms" onclick={(event) => follow(event, '/terms')}>Terms</a><a href="https://sociobot.in" rel="external">Built by Param Factory <span class="sr-only">(opens Sociobot)</span></a></nav><small>m1-public-proof-sandbox</small></footer>
