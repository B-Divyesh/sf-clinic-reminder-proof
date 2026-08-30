import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { expect, test } from '@playwright/test';
import { createFreshTestClient } from '../../scripts/fresh-client-identity.mjs';

const exec = promisify(execFile);

test.beforeEach(async ({ page }) => {
  await page.context().setExtraHTTPHeaders({ 'x-forwarded-for': createFreshTestClient() });
  await page.goto('/?demo=1');
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today’s sample reminders');
});

async function cargoClaim(name: string): Promise<string> {
  const { stdout, stderr } = await exec('cargo', [
    'test',
    '--manifest-path',
    'services/api/Cargo.toml',
    name,
    '--',
    '--exact',
    '--nocapture'
  ], { cwd: process.cwd(), timeout: 120_000 });
  return `${stdout}\n${stderr}`;
}

test('@claim:ciam-sign-in Sign in uses the shared Sociobot customer account.', async ({ page, request }) => {
  const configResponse = await request.get('/api/v1/auth/config');
  expect(configResponse.ok()).toBe(true);
  await expect(configResponse.json()).resolves.toMatchObject({
    tenant_id: '35c6fe40-0ec0-46b6-98c6-213ad4de6650',
    client_id: '25c704f4-465a-47af-80ab-2c489466b697',
    authority: 'https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/'
  });
  await page.goto('/sign-in');
  await expect(page).toHaveTitle('Sign in — Reminder Proof');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Sign in to manage a clinic');
  await expect(page.getByRole('button', { name: 'Sign in with Microsoft' })).toBeEnabled();
  const output = await cargoClaim('auth::tests::m2_claim_ciam_contract_rejects_wrong_registered_claims');
  expect(output).toContain('m2_claim_ciam_contract_rejects_wrong_registered_claims ... ok');
});

test('@claim:tenant-isolation Clinic data stays inside its organization.', async () => {
  const output = await cargoClaim('clinic::tests::m2_claim_tenant_roles_and_onboarding_survive_restart');
  expect(output).toContain('m2_claim_tenant_roles_and_onboarding_survive_restart ... ok');
});

test('@claim:durable-onboarding Clinic and location settings remain after sign-out and sign-in.', async ({ page }) => {
  for (const route of ['/onboarding/clinic', '/onboarding/location', '/onboarding/staff']) {
    await page.goto(route);
    await expect(page).toHaveTitle(/Reminder Proof$/);
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  }
  const output = await cargoClaim('clinic::tests::m2_claim_tenant_roles_and_onboarding_survive_restart');
  expect(output).toContain('m2_claim_tenant_roles_and_onboarding_survive_restart ... ok');
  const migration = await cargoClaim('clinic::tests::m2_claim_reversible_account_migration_round_trips');
  expect(migration).toContain('m2_claim_reversible_account_migration_round_trips ... ok');
});

test('@claim:subscription-price Clinic costs $79 per location each month.', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('Clinic costs $79 per location each month.')).toBeVisible();
  await page.goto('/terms');
  await expect(page.getByText('The Clinic plan is $79 per location each month.')).toBeVisible();
  const output = await cargoClaim('clinic::tests::managed_claim_billing_checkout_and_return_activates_subscription');
  expect(output).toContain('managed_claim_billing_checkout_and_return_activates_subscription ... ok');
});

test('@claim:data-export An owner can export the clinic’s stored data.', async () => {
  const output = await cargoClaim('clinic::tests::m2_claim_export_and_seven_day_deletion_are_owner_controlled');
  expect(output).toContain('m2_claim_export_and_seven_day_deletion_are_owner_controlled ... ok');
});

test('@claim:account-deletion An owner can schedule account deletion with a seven-day recovery window.', async ({ page }) => {
  await page.goto('/app/settings/privacy');
  await expect(page).toHaveTitle('Clinic data controls — Reminder Proof');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Clinic data controls');
  const output = await cargoClaim('clinic::tests::m2_claim_export_and_seven_day_deletion_are_owner_controlled');
  expect(output).toContain('m2_claim_export_and_seven_day_deletion_are_owner_controlled ... ok');
});
