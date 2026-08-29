import { randomUUID } from 'node:crypto';

/**
 * Create an isolated client identity for a rate-limit probe.
 *
 * The documentation-only IPv6 prefix is accepted by the API's IP parser but
 * is never routed on the public internet. Six UUID groups leave 96 bits of
 * entropy, so a later clean verification cannot reuse a one-hour demo bucket
 * from an earlier run.
 */
export function createFreshTestClient(uuid = randomUUID) {
  const groups = uuid().replaceAll('-', '').match(/[0-9a-f]{4}/gi);
  if (!groups || groups.length !== 8) {
    throw new Error('rate-limit test identity generator must return a UUID');
  }
  return `2001:db8:${groups.slice(0, 6).join(':')}`;
}
