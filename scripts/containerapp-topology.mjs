import { buildShaFromImage } from './deployment-identity.mjs';

/**
 * The durable-storage topology is deliberately kept separate from image
 * selection. Image-only Container Apps updates replace the template and can
 * otherwise discard these mounts, so every rollout composes this file's
 * topology into the template that is sent to Azure.
 */
export function validateTopology(document) {
  const template = document?.properties?.template;
  if (!template || typeof template !== 'object') {
    throw new Error('deployment topology must contain properties.template');
  }

  if (template.scale?.minReplicas !== 1 || template.scale?.maxReplicas !== 1) {
    throw new Error('deployment topology must set minReplicas and maxReplicas to 1');
  }

  const expectedVolumes = [
    { name: 'clinic-data', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-data' },
    { name: 'clinic-backups', storageType: 'AzureFile', storageName: 'clinic-reminder-proof-backups' }
  ];
  for (const expected of expectedVolumes) {
    const volume = template.volumes?.find((item) => item.name === expected.name);
    if (!volume || volume.storageType !== expected.storageType || volume.storageName !== expected.storageName) {
      throw new Error(`deployment topology must configure ${expected.name} as its expected Azure Files share`);
    }
  }

  const app = template.containers?.find((container) => container.name === 'app');
  if (!app) throw new Error('deployment topology must configure the app container');
  for (const expected of [
    { volumeName: 'clinic-data', mountPath: '/durable' },
    { volumeName: 'clinic-backups', mountPath: '/backups' }
  ]) {
    const mount = app.volumeMounts?.find((item) => item.volumeName === expected.volumeName);
    if (!mount || mount.mountPath !== expected.mountPath) {
      throw new Error(`deployment topology must mount ${expected.volumeName} at ${expected.mountPath}`);
    }
  }

  return template;
}

/**
 * Build the smallest ARM patch that retains the live app settings while
 * replacing every revision-template field that controls safety. This makes an
 * image rollout deterministic even if the prior template was incomplete.
 */
export function buildTopologyPatch(currentApp, topologyDocument, image) {
  // A short or mutable tag prevents the runtime build identity from being
  // proven. Every deploy must name the exact source commit it was built from.
  buildShaFromImage(image);

  const topology = validateTopology(topologyDocument);
  const currentTemplate = currentApp?.properties?.template;
  const currentContainer = currentTemplate?.containers?.find((container) => container.name === 'app');
  if (!currentTemplate || !currentContainer) {
    throw new Error('the current Container App must contain an app container');
  }

  return {
    properties: {
      template: {
        ...currentTemplate,
        containers: currentTemplate.containers.map((container) => {
          if (container.name !== 'app') return container;
          return {
            ...currentContainer,
            name: 'app',
            image,
            volumeMounts: topology.containers.find((item) => item.name === 'app').volumeMounts
          };
        }),
        volumes: topology.volumes,
        scale: {
          ...currentTemplate.scale,
          ...topology.scale
        }
      }
    }
  };
}

function revisionImage(revision) {
  return revision?.properties?.template?.containers?.find((container) => container.name === 'app')?.image;
}

/**
 * Inspect the control-plane state after a rollout. Azure can keep an
 * unhealthy latest revision at 100% in its traffic metadata while requests
 * fall back to the previous healthy revision. A rollout is complete only when
 * the exact healthy revision is also the sole declared traffic target.
 */
export function inspectRollout(app, revisions, image) {
  if (!Array.isArray(revisions)) throw new Error('Container App revisions must be an array');

  const readyRevision = revisions.find((revision) =>
    revision.name === app?.properties?.latestReadyRevisionName
      && revision.properties?.healthState === 'Healthy'
      && revisionImage(revision) === image
  );
  const trafficRevisions = revisions.filter(
    (revision) => revision.properties?.active && revision.properties?.trafficWeight > 0
  );
  const trafficConverged = Boolean(
    readyRevision
      && trafficRevisions.length === 1
      && trafficRevisions[0].name === readyRevision.name
      && trafficRevisions[0].properties?.trafficWeight === 100
  );

  return {
    readyRevision,
    trafficRevisions,
    trafficConverged
  };
}
