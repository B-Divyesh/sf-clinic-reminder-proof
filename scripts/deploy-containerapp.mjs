import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { buildTopologyPatch, inspectRollout, validateTopology } from './containerapp-topology.mjs';
import { fetchPublicBuildIdentity } from './deployment-identity.mjs';
import {
  assertDeploymentImageMatchesSource,
  assertReleaseCheckoutReady,
  resolveCheckedOutSourceCommit
} from './source-commit.mjs';

const resourceGroup = process.env.REMINDER_PROOF_RESOURCE_GROUP ?? 'sociobot';
const appName = process.env.REMINDER_PROOF_APP_NAME ?? 'sf-clinic-reminder-proof';
const liveUrl = (process.env.REMINDER_PROOF_LIVE_URL ?? 'https://clinic-reminder-proof.sociobot.in').replace(/\/$/, '');
const apiVersion = '2025-07-01';
const deploymentTimeoutMs = Number.parseInt(process.env.DEPLOYMENT_TIMEOUT_MS ?? '600000', 10);
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

function wait(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

if (!image || image.startsWith('--')) {
  fail('pass --image <registry/image:tag> or set REMINDER_PROOF_IMAGE');
}
const checkedOutCandidate = resolveCheckedOutSourceCommit();
assertReleaseCheckoutReady(checkedOutCandidate);
const expectedBuildSha = assertDeploymentImageMatchesSource(image, checkedOutCandidate);

const topology = JSON.parse(await readFile(new URL('../deployment/containerapp.json', import.meta.url), 'utf8'));
const currentApp = JSON.parse(
  azure(['containerapp', 'show', '--resource-group', resourceGroup, '--name', appName, '--output', 'json'])
);
const patch = buildTopologyPatch(currentApp, topology, image);

if (currentApp.properties?.configuration?.activeRevisionsMode !== 'Single') {
  fail('the app must use single revision mode for the SQLite consistency boundary');
}

if (dryRun) {
  console.log(JSON.stringify(patch, null, 2));
  process.exit(0);
}

// PATCH is intentional: ingress, custom domains, managed identity, and other
// app-level settings stay managed by the factory. The revision template is
// fully composed above so image changes cannot drop durable mounts or scale.
const endpoint = `${currentApp.id}?api-version=${apiVersion}`;
azure(['rest', '--method', 'PATCH', '--uri', endpoint, '--body', JSON.stringify(patch), '--output', 'none']);

const deadline = Date.now() + deploymentTimeoutMs;
let servingRevision;
let lastPublicIdentityError;
while (Date.now() < deadline) {
  const app = JSON.parse(
    azure(['containerapp', 'show', '--resource-group', resourceGroup, '--name', appName, '--output', 'json'])
  );
  const revisions = JSON.parse(
    azure(['containerapp', 'revision', 'list', '--resource-group', resourceGroup, '--name', appName, '--output', 'json'])
  );
  const rollout = inspectRollout(app, revisions, image);
  if (rollout.readyRevision) {
    validateTopology({ properties: { template: rollout.readyRevision.properties?.template } });
    if (rollout.latestRevisionConverged && rollout.readyRevision.properties?.replicas === 1) {
      validateTopology({ properties: { template: app.properties?.template } });
      try {
        await fetchPublicBuildIdentity(liveUrl, expectedBuildSha);
        servingRevision = rollout.readyRevision;
        break;
      } catch (error) {
        lastPublicIdentityError = error;
      }
    }
  }
  await wait(10_000);
}

if (!servingRevision) {
  const publicIdentity = lastPublicIdentityError ? ` Public identity check: ${lastPublicIdentityError.message}` : '';
  fail(`the latest target revision did not become healthy, receive sole 100% traffic, and serve its exact build before the deployment timeout.${publicIdentity}`);
}

console.log(JSON.stringify({
  app: appName,
  image,
  revision: servingRevision.name,
  trafficWeight: 100,
  message: 'Applied the checked-in durable topology and confirmed its healthy revision has all traffic. Run verify:deployment with the expected build SHA.'
}, null, 2));
