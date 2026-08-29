import { execFile } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import { expect, test } from '@playwright/test';

const exec = promisify(execFile);

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

test('@claim:managed-provider-fallback-receipt A rejected approved channel falls back and a signed receipt records delivery.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_provider_fallback_and_receipt_is_observable');
  expect(output).toContain('managed_claim_provider_fallback_and_receipt_is_observable ... ok');
});

test('@claim:signed-calendar-intake Signed calendar intake rejects altered signatures and upserts duplicate source IDs.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_signed_calendar_intake_is_authenticated_and_idempotent');
  expect(output).toContain('managed_claim_signed_calendar_intake_is_authenticated_and_idempotent ... ok');
});

test('@claim:approved-whatsapp-dispatch Approved WhatsApp dispatch uses the configured template and records its receipt.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_approved_whatsapp_uses_template_and_records_receipt');
  expect(output).toContain('managed_claim_approved_whatsapp_uses_template_and_records_receipt ... ok');
});

test('@claim:twilio-receipt-verification Twilio callbacks require valid signatures and ignore replayed receipt IDs.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_twilio_receipt_verification_is_replay_safe');
  expect(output).toContain('managed_claim_twilio_receipt_verification_is_replay_safe ... ok');
});

test('@claim:resend-receipt-verification Resend callbacks require valid Svix signatures and ignore replayed receipt IDs.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_resend_receipt_verification_is_replay_safe');
  expect(output).toContain('managed_claim_resend_receipt_verification_is_replay_safe ... ok');
});

test('@claim:managed-secret-encryption Provider secrets and destinations remain encrypted outside the authorized adapter.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_secrets_and_destinations_are_encrypted_and_adapter_scoped');
  expect(output).toContain('managed_claim_secrets_and_destinations_are_encrypted_and_adapter_scoped ... ok');
});

test('@claim:managed-data-minimisation Managed writes reject clinical fields and keep them out of storage and export.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_clinical_fields_are_rejected_at_every_json_write');
  expect(output).toContain('managed_claim_clinical_fields_are_rejected_at_every_json_write ... ok');
});

test('@claim:managed-billing-return Sociobot checkout and a valid return activate the clinic subscription.', async () => {
  const output = await cargoClaim('clinic::tests::managed_claim_billing_checkout_and_return_activates_subscription');
  expect(output).toContain('managed_claim_billing_checkout_and_return_activates_subscription ... ok');
});

test('@claim:managed-storage-recovery Every saved clinic change has a durable matching key and a 30-day recovery pair.', async () => {
  const output = await cargoClaim('clinic::tests::managed_storage_recovery_claim');
  expect(output).toContain('managed_storage_recovery_claim ... ok');
});

test('@claim:single-replica-durable-topology Production configuration has one replica and mounts both recovery shares.', async () => {
  const deployment = JSON.parse(await readFile('deployment/containerapp.json', 'utf8')) as {
    properties: { template: { scale: { minReplicas: number; maxReplicas: number }; volumes: Array<{ name: string; storageName: string; storageType: string }>; containers: Array<{ volumeMounts: Array<{ volumeName: string; mountPath: string }> }> } };
  };
  const template = deployment.properties.template;
  expect(template.scale).toMatchObject({ minReplicas: 1, maxReplicas: 1 });
  expect(template.volumes).toEqual(expect.arrayContaining([
    { name: 'clinic-data', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-data' },
    { name: 'clinic-backups', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-backups' }
  ]));
  expect(template.containers[0].volumeMounts).toEqual(expect.arrayContaining([
    { volumeName: 'clinic-data', mountPath: '/durable' },
    { volumeName: 'clinic-backups', mountPath: '/backups' }
  ]));
});
