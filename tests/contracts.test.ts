import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { describe, expect, test } from 'vitest';
import { effectiveConsent, foldReminderOutcome, stateCopy } from '../apps/web/src/lib/reminder';

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
    for (const claim of claims) {
      expect(claim.claim.length).toBeGreaterThan(10);
      expect(claim.test).toContain(`@claim:${claim.id}`);
      expect(claim.sandbox.length).toBeGreaterThan(30);
    }
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
