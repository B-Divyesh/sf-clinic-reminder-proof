import { spawnSync } from 'node:child_process';
import { validateTopology } from './containerapp-topology.mjs';
import { assertImageMatchesBuildSha, fetchPublicBuildIdentity, normalizeBuildSha } from './deployment-identity.mjs';

const resourceGroup = process.env.REMINDER_PROOF_RESOURCE_GROUP ?? 'sociobot';
const appName = process.env.REMINDER_PROOF_APP_NAME ?? 'sf-clinic-reminder-proof';
const liveUrl = (process.env.REMINDER_PROOF_LIVE_URL ?? 'https://clinic-reminder-proof.sociobot.in').replace(/\/$/, '');
const expectedBuildSha = process.env.EXPECTED_BUILD_SHA;

function fail(message) {
  throw new Error(`Deployment verification failed: ${message}`);
}

function azure(args) {
  const result = spawnSync('az', args, { encoding: 'utf8' });
  if (result.status !== 0) fail(result.stderr.trim() || `az ${args.join(' ')} exited ${result.status}`);
  return result.stdout;
}

function requireEqual(actual, expected, description) {
  if (actual !== expected) fail(`${description}; expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

if (!expectedBuildSha) fail('set EXPECTED_BUILD_SHA to the full immutable source commit before verifying production');
const expected = normalizeBuildSha(expectedBuildSha, 'EXPECTED_BUILD_SHA');

const app = JSON.parse(azure(['containerapp', 'show', '--resource-group', resourceGroup, '--name', appName, '--output', 'json']));
const template = app.properties?.template;
validateTopology({ properties: { template } });
requireEqual(template?.scale?.minReplicas, 1, 'minimum replica count');
requireEqual(template?.scale?.maxReplicas, 1, 'maximum replica count');

const volumes = template?.volumes ?? [];
for (const expected of [
  { name: 'clinic-data', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-data' },
  { name: 'clinic-backups', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-backups' }
]) {
  const volume = volumes.find((item) => item.name === expected.name);
  if (!volume) fail(`missing ${expected.name} Azure Files volume`);
  requireEqual(volume.storageType, expected.storageType, `${expected.name} storage type`);
  requireEqual(volume.storageName, expected.storageName, `${expected.name} storage binding`);
}

const mounts = template?.containers?.find((container) => container.name === 'app')?.volumeMounts ?? [];
for (const expected of [
  { volumeName: 'clinic-data', mountPath: '/durable' },
  { volumeName: 'clinic-backups', mountPath: '/backups' }
]) {
  const mount = mounts.find((item) => item.volumeName === expected.volumeName);
  if (!mount) fail(`missing ${expected.volumeName} mount`);
  requireEqual(mount.mountPath, expected.mountPath, `${expected.volumeName} mount path`);
}

const revisions = JSON.parse(azure(['containerapp', 'revision', 'list', '--resource-group', resourceGroup, '--name', appName, '--output', 'json']));
const serving = revisions.filter((revision) => revision.properties?.active && revision.properties?.trafficWeight > 0);
requireEqual(serving.length, 1, 'traffic-bearing revision count');
requireEqual(serving[0]?.name, app.properties?.latestReadyRevisionName, 'serving revision');
requireEqual(serving[0]?.properties?.trafficWeight, 100, 'serving revision traffic weight');
requireEqual(serving[0]?.properties?.replicas, 1, 'serving replica count');
validateTopology({ properties: { template: serving[0]?.properties?.template } });

const image = serving[0]?.properties?.template?.containers?.find((container) => container.name === 'app')?.image;
try {
  assertImageMatchesBuildSha(image, expected);
  await fetchPublicBuildIdentity(liveUrl, expected);
} catch (error) {
  fail(error.message);
}

const randomOctet = () => Math.floor(Math.random() * 254) + 1;
const clientIp = `198.18.${randomOctet()}.${randomOctet()}`;
const rateStatuses = [];
let retryAfter = null;
for (let request = 0; request < 6; request += 1) {
  const response = await fetch(`${liveUrl}/api/v1/demo/workspaces`, {
    method: 'POST',
    headers: { 'x-forwarded-for': `${clientIp}, 203.0.113.${request + 1}` }
  });
  rateStatuses.push(response.status);
  if (request === 5) retryAfter = response.headers.get('retry-after');
}
if (rateStatuses.slice(0, 5).some((status) => status !== 200)) fail(`first five demo creations returned ${rateStatuses.join(', ')}`);
requireEqual(rateStatuses[5], 429, `sixth demo creation status for ${clientIp}`);
if (!retryAfter || Number(retryAfter) <= 0) fail(`sixth demo creation Retry-After header was ${JSON.stringify(retryAfter)}`);

console.log(JSON.stringify({
  app: appName,
  revision: serving[0].name,
  image,
  replicas: serving[0].properties.replicas,
  buildSha: expected,
  rateStatuses,
  retryAfter
}, null, 2));
