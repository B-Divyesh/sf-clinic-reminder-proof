import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { describe, expect, test } from 'vitest';
import { effectiveConsent, foldReminderOutcome, stateCopy } from '../apps/web/src/lib/reminder';
import { buildTopologyPatch } from '../scripts/containerapp-topology.mjs';

const repositoryRoot = fileURLToPath(new URL('../', import.meta.url));

async function readRepositoryFile(path: string): Promise<string> {
  return readFile(`${repositoryRoot}${path}`, 'utf8');
}

function parseHex(value: string): [number, number, number] {
  const hex = value.replace('#', '');
  return [0, 2, 4].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16)) as [
    number,
    number,
    number
  ];
}

function luminance(hex: string): number {
  const channels = parseHex(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(first: string, second: string): number {
  const [lighter, darker] = [luminance(first), luminance(second)].sort((a, b) => b - a);
  return (lighter + 0.05) / (darker + 0.05);
}

function token(block: string, name: string): string {
  const match = block.match(new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{6})`));
  if (!match) throw new Error(`Missing hexadecimal token --${name}`);
  return match[1];
}

describe('planning scaffold contracts', () => {
  test('the component inventory has 12–20 unique components with states', async () => {
    const inventory = JSON.parse(
      await readRepositoryFile('packages/design-system/component-inventory.json')
    ) as Array<{ name: string; firstMilestone: string; states: string[] }>;

    expect(inventory).toHaveLength(20);
    expect(new Set(inventory.map(({ name }) => name)).size).toBe(inventory.length);
    expect(inventory.every(({ firstMilestone }) => /^M[1-5]$/.test(firstMilestone))).toBe(true);
    expect(inventory.every(({ states }) => states.length >= 2)).toBe(true);
  });

  test('light and dark text tokens meet WCAG contrast contracts', async () => {
    const css = await readRepositoryFile('packages/design-system/tokens.css');
    const light = css.match(/:root\s*\{([\s\S]*?)\}/)?.[1];
    const dark = css.match(/\[data-theme="dark"\]\s*\{([\s\S]*?)\}/)?.[1];

    expect(light, 'light token block').toBeTruthy();
    expect(dark, 'dark token block').toBeTruthy();

    for (const block of [light!, dark!]) {
      expect(contrast(token(block, 'color-text'), token(block, 'color-bg'))).toBeGreaterThanOrEqual(4.5);
      expect(contrast(token(block, 'color-muted'), token(block, 'color-bg'))).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(token(block, 'color-accent'), token(block, 'color-accent-contrast'))
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  test('product claims are unique and point to exact claim tags', async () => {
    const claims = JSON.parse(await readRepositoryFile('.factory/claims.json')) as Array<{
      id: string;
      claim: string;
      test: string;
      sandbox: string;
    }>;

    expect(claims.length).toBeGreaterThanOrEqual(9);
    expect(new Set(claims.map(({ id }) => id)).size).toBe(claims.length);
    const claimSpecs = [
      await readRepositoryFile('tests/e2e/m1-claims.spec.ts'),
      await readRepositoryFile('tests/e2e/managed-claims.spec.ts')
    ].join('\n');
    for (const claim of claims) {
      expect(claim.claim.length).toBeGreaterThan(10);
      expect(claim.test).toContain(`@claim:${claim.id}`);
      expect(claim.sandbox.length).toBeGreaterThan(30);
      const escapedId = claim.id.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      expect(claimSpecs.match(new RegExp(`@claim:${escapedId}(?![a-z0-9-])`, 'g'))).toHaveLength(1);
    }
  });

  test('catalog description is verb-first, plain, and no longer than 120 characters', async () => {
    const description = (await readRepositoryFile('.factory/catalog-description.txt')).trim();
    expect(description.length).toBeLessThanOrEqual(120);
    expect(description).toMatch(/^Track\b/);
    expect(description).not.toMatch(/seamless|effortless|robust|powerful|intuitive|reimagine|supercharge|delightful|journey|ecosystem|AI-powered/i);
  });

  test('README cumulative plain-words repairs keep each reviewed sentence short and concrete', async () => {
    const readme = await readRepositoryFile('README.md');
    const required = [
      'Reminder Proof records each appointment reminder outcome for independent clinics.',
      'It shows source details, consent, messaging-provider evidence, and the staff owner.',
      'Advance the sample reminders and inspect their evidence.',
      'Assign or resolve the sample exception, undo a resolution, and reset the sample clinic.',
      'Reminder Proof checks recorded consent before sending.',
      'It records messaging-provider receipts and opens a shared exception when delivery proof is missing.',
      'A protected browser cookie keeps the sample available for 24 hours, including after a server restart.',
      'A service on this site keeps demo sessions separate from clinic data.',
      'A signed calendar/EMR connection stores each appointment once, even when it receives the same update twice.',
      'The site includes pulse-ledger art, a favicon, a touch icon, a social card, and self-hosted Instrument Sans and Fragment Mono fonts.',
      'It includes rate limits, health checks, and machine-readable metrics.',
      'Each saved change writes a matching durable database and key under `DURABLE_DIR`.',
      'A daily recovery copy is kept under `BACKUP_DIR` for 30 days.',
      'The container mounts separate durable and backup shares at `/durable` and `/backups`.',
      'The production image refuses to start when either required share is missing.',
      'The application runs without root privileges.'
    ];
    for (const sentence of required) {
      expect(readme).toContain(sentence);
      expect(sentence.replace(/[`“”]/g, '').split(/\s+/).length).toBeLessThanOrEqual(22);
    }
    for (const removed of [
      'Reminder Proof gives independent clinic teams a clear proof trail',
      'You can advance sample reminders, inspect provider evidence',
      'A Rust/axum same-origin API with isolated demo cookies',
      'Each acknowledged workspace mutation synchronously checkpoints',
      'Separate durable Azure Files shares mount directly',
      'Try the public sandbox',
      'HttpOnly, Secure browser cookie',
      'A same-origin service protects',
      'idempotent appointment upserts',
      'Original hand-authored',
      'Delivery-provider fees',
      'approved delivery providers',
      'consent, provider evidence',
      'It records provider receipts'
    ]) expect(readme).not.toContain(removed);

    expect(readme).toContain('## Try the demo');
  });

  test('review-three public copy uses direct 404 and messaging-provider terms', async () => {
    const app = await readRepositoryFile('apps/web/src/App.svelte');
    expect(app).toContain("heading: 'Page not found'");
    expect(app).toContain('>Page not found</h1>');
    expect(app).toContain('Build {shortBuildSha}');
    for (const ambiguous of [
      'each attempt, provider result',
      'Every provider result',
      '>Provider result<',
      '>Provider mode<',
      'with provider evidence',
      'Simulated provider events',
      'simulated provider attempt',
      'Delivery-provider',
      '<h2>Approved provider',
      '>Provider credential<',
      '>Save provider<',
      '>Provider proof<'
    ]) expect(app).not.toContain(ambiguous);
  });

  test('@regression:qa12-01 image deployment reapplies durable mounts and the single-replica boundary', async () => {
    const topology = JSON.parse(await readRepositoryFile('deployment/containerapp.json'));
    const brokenTemplate = {
      properties: {
        template: {
          containers: [{
            name: 'app',
            image: 'sociobotregistry.azurecr.io/sf-clinic-reminder-proof:a95a64b6f1cc',
            env: [{ name: 'PORT', value: '8080' }],
            resources: { cpu: 0.5, memory: '1Gi' }
          }],
          scale: { minReplicas: 1, maxReplicas: 3, cooldownPeriod: 300 },
          volumes: null
        }
      }
    };

    const image = 'sociobotregistry.azurecr.io/sf-clinic-reminder-proof:0123456789abcdef0123456789abcdef01234567';
    const template = buildTopologyPatch(brokenTemplate, topology, image).properties.template;
    expect(template.scale).toMatchObject({ minReplicas: 1, maxReplicas: 1 });
    expect(template.volumes).toEqual([
      { name: 'clinic-data', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-data' },
      { name: 'clinic-backups', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-backups' }
    ]);
    expect(template.containers[0]).toMatchObject({
      image,
      env: [{ name: 'PORT', value: '8080' }],
      resources: { cpu: 0.5, memory: '1Gi' },
      volumeMounts: [
        { volumeName: 'clinic-data', mountPath: '/durable' },
        { volumeName: 'clinic-backups', mountPath: '/backups' }
      ]
    });
  });

  test('the container build context excludes Git metadata', async () => {
    expect(await readRepositoryFile('.dockerignore')).toMatch(/^\.git$/m);
  });
});

describe('M1 reminder domain contracts', () => {
  test('an opt-out wins over any recorded consent', () => {
    expect(effectiveConsent(['allowed', 'blocked'])).toBe('blocked');
    expect(effectiveConsent(['unknown'])).toBe('blocked');
  });

  test('delivery and cancellation fold with safe precedence', () => {
    expect(foldReminderOutcome([{ kind: 'attempt', outcome: 'Delivered' }])).toBe('delivered');
    expect(
      foldReminderOutcome([
        { kind: 'attempt', outcome: 'Delivered' },
        { kind: 'source', outcome: 'Cancelled' }
      ])
    ).toBe('cancelled');
    expect(foldReminderOutcome([{ kind: 'consent', outcome: 'Blocked' }])).toBe('exception');
  });

  test('state copy keeps provider acceptance separate from delivery proof', () => {
    expect(stateCopy.providerPending).toContain('Delivery is not confirmed');
    expect(stateCopy.exhausted).toContain('Assign someone');
  });
});
