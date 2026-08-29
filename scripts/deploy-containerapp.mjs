import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { buildTopologyPatch } from './containerapp-topology.mjs';

const resourceGroup = process.env.REMINDER_PROOF_RESOURCE_GROUP ?? 'sociobot';
const appName = process.env.REMINDER_PROOF_APP_NAME ?? 'sf-clinic-reminder-proof';
const apiVersion = '2025-07-01';
const args = process.argv.slice(2);
const imageIndex = args.indexOf('--image');
const image = imageIndex >= 0 ? args[imageIndex + 1] : process.env.REMINDER_PROOF_IMAGE;
const dryRun = args.includes('--dry-run');

function fail(message) {
  throw new Error(`Container App deployment failed: ${message}`);
}

function azure(command) {
  const result = spawnSync('az', command, { encoding: 'utf8' });
  if (result.status !== 0) fail(result.stderr.trim() || `az ${command.join(' ')} exited ${result.status}`);
  return result.stdout;
}

if (!image || image.startsWith('--')) {
  fail('pass --image <registry/image:tag> or set REMINDER_PROOF_IMAGE');
}

const topology = JSON.parse(await readFile(new URL('../deployment/containerapp.json', import.meta.url), 'utf8'));
const currentApp = JSON.parse(
  azure(['containerapp', 'show', '--resource-group', resourceGroup, '--name', appName, '--output', 'json'])
);
const patch = buildTopologyPatch(currentApp, topology, image);

if (dryRun) {
  console.log(JSON.stringify(patch, null, 2));
  process.exit(0);
}

// PATCH is intentional: ingress, custom domains, managed identity, and other
// app-level settings stay managed by the factory. The revision template is
// fully composed above so image changes cannot drop durable mounts or scale.
const endpoint = `${currentApp.id}?api-version=${apiVersion}`;
const response = JSON.parse(
  azure(['rest', '--method', 'PATCH', '--uri', endpoint, '--body', JSON.stringify(patch), '--output', 'json'])
);
const template = response.properties?.template;
if (template?.containers?.find((container) => container.name === 'app')?.image !== image) {
  fail('Azure did not accept the requested container image');
}

console.log(JSON.stringify({
  app: appName,
  image,
  revision: response.properties?.latestRevisionName,
  message: 'Applied the checked-in durable topology with the image rollout. Run verify:deployment after readiness.'
}, null, 2));
