import { randomUUID } from 'node:crypto';

/**
 * Create an unpredictable documentation-only forwarding prefix for a
 * rate-limit probe.
 *
 * Local tests use it as the final, ingress-derived hop. Public verification
 * varies it only as a caller-controlled prefix; the application must instead
 * use the final hop that the trusted ingress appends for the physical client.
 */
export function createFreshTestClient(uuid = randomUUID) {
  const groups = uuid().replaceAll('-', '').match(/[0-9a-f]{4}/gi);
  if (!groups || groups.length !== 8) {
    throw new Error('rate-limit test identity generator must return a UUID');
  }
  return `2001:db8:${groups.slice(0, 6).join(':')}`;
}
