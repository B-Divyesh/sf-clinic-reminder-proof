<script lang="ts">
  export let label = 'A sample proof ledger with source, consent, delivery, reply, and exception markers.';
</script>

<svg class="pulse-ledger-art" viewBox="0 0 720 430" role="img" aria-label={label}>
  <defs>
    <linearGradient id="ledger-wash" x1="0" x2="1" y1="0" y2="1">
      <stop offset="0" stop-color="#e3f5f1" />
      <stop offset="1" stop-color="#f9f3e7" />
    </linearGradient>
  </defs>
  <rect width="720" height="430" fill="url(#ledger-wash)" />
  <g stroke="#a9c1c1" stroke-width="1">
    <path d="M0 86H720M0 150H720M0 214H720M0 278H720M0 342H720" />
    <path d="M78 0V430M202 0V430M326 0V430M450 0V430M574 0V430M698 0V430" opacity=".45" />
  </g>
  <g fill="#fbfdfc" fill-opacity=".86" stroke="#78a6a5">
    <path d="M50 75h178l22 22v55H50z" />
    <path d="M236 138h175l24 23v55H236z" />
    <path d="M422 202h178l24 22v55H422z" />
    <path d="M516 266h155l20 21v55H516z" />
  </g>
  <g fill="none" stroke="#006a73" stroke-width="3" stroke-linecap="round">
    <path d="M83 128h98M267 191h91M454 254h105M548 318h77" opacity=".65" />
    <path d="M203 128H285V191H450V254H544V318" />
  </g>
  <g>
    <circle cx="203" cy="128" r="11" fill="#216e4a" stroke="#fff" stroke-width="5" />
    <rect x="275" y="181" width="20" height="20" rx="2" fill="#a12c3f" stroke="#fff" stroke-width="5" />
    <circle cx="450" cy="254" r="12" fill="#006a73" stroke="#fff" stroke-width="5" />
    <path d="M544 303l13 15-13 15-13-15z" fill="#8a4f00" stroke="#fff" stroke-width="5" />
    <path d="M626 307c10 0 17 7 17 16 0 10-7 17-17 17-4 0-7-1-10-3l-9 4 3-9c-1-3-1-5-1-8 0-10 7-17 17-17z" fill="#5b4ab4" stroke="#fff" stroke-width="5" />
  </g>
</svg>
