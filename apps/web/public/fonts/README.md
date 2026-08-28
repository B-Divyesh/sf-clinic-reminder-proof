# Self-hosted type assets

The web build self-hosts two OFL font families from npm packages; Vite emits the WOFF2 files into this product's `dist/assets/` directory.

- Instrument Sans variable — upstream: https://github.com/Instrument/instrument-sans — SIL Open Font License 1.1. Package: `@fontsource-variable/instrument-sans` 5.3.0.
- Fragment Mono regular — upstream: https://github.com/weiweihuanghuang/fragment-mono — SIL Open Font License 1.1. Package: `@fontsource/fragment-mono` 5.2.6.

Only the CSS imports used by `apps/web/src/styles.css` are bundled. No font is fetched from a CDN at runtime.
