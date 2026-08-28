<svelte:head>
  <title>Reminder Proof — Product skeleton</title>
</svelte:head>

<a class="skip-link" href="#main">Skip to main content</a>

<header class="site-header">
  <a class="wordmark" href="/" aria-label="Reminder Proof home">
    <span class="wordmark-mark" aria-hidden="true"><i></i><i></i><i></i></span>
    Reminder Proof
  </a>
  <span class="build-label">Planning scaffold</span>
</header>

<main id="main" class="shell">
  <section class="intro" aria-labelledby="page-title">
    <p class="eyebrow">M1 builder starting point</p>
    <h1 id="page-title" tabindex="-1">The product skeleton is ready.</h1>
    <p class="lede">
      The venture contract, design tokens, component inventory, web entry point, and API health
      service are wired for the first product milestone.
    </p>
  </section>

  <section class="ledger-sheet" aria-labelledby="included-title">
    <div class="ledger-heading">
      <div>
        <p class="eyebrow">Repository contract</p>
        <h2 id="included-title">What M1 starts with</h2>
      </div>
      <span class="status">Scaffold only</span>
    </div>

    <ol class="ledger-list">
      <li>
        <span class="marker" aria-hidden="true"></span>
        <div><strong>Product plan</strong><span>PRD, architecture, milestones, and risks</span></div>
        <code>.factory/plan.md</code>
      </li>
      <li>
        <span class="marker" aria-hidden="true"></span>
        <div><strong>Visual thesis</strong><span>Translucent pulse ledger system</span></div>
        <code>.factory/design.md</code>
      </li>
      <li>
        <span class="marker" aria-hidden="true"></span>
        <div><strong>Acceptance contract</strong><span>Nine testable M1 claims</span></div>
        <code>.factory/claims.json</code>
      </li>
      <li>
        <span class="marker" aria-hidden="true"></span>
        <div><strong>Build foundation</strong><span>Svelte, Vite, Rust, and CI</span></div>
        <code>npm run build</code>
      </li>
    </ol>
  </section>

  <p class="boundary">
    This page is a development scaffold, not the M1 demo. It sends no reminders and stores no
    clinic data.
  </p>
</main>
