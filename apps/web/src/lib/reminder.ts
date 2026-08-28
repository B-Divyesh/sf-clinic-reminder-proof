export type ReminderEvent = {
  kind: 'source' | 'consent' | 'attempt' | 'response' | 'exception';
  outcome: string;
};

export function effectiveConsent(statuses: Array<'allowed' | 'blocked' | 'unknown'>): 'allowed' | 'blocked' {
  return statuses.includes('blocked') ? 'blocked' : statuses.includes('allowed') ? 'allowed' : 'blocked';
}

export function foldReminderOutcome(events: ReminderEvent[]): 'delivered' | 'exception' | 'cancelled' | 'scheduled' {
  if (events.some((event) => event.outcome === 'Cancelled')) return 'cancelled';
  if (events.some((event) => event.kind === 'attempt' && event.outcome === 'Delivered')) return 'delivered';
  if (events.some((event) => event.kind === 'exception' || event.outcome === 'Blocked')) return 'exception';
  return 'scheduled';
}

export const stateCopy = {
  empty: 'No reminders are due in this range.',
  providerPending: 'The provider accepted this attempt. Delivery is not confirmed yet.',
  exhausted: 'No allowed channel delivered this reminder. Assign someone to follow up.',
  offline: 'You’re offline. Sending and resolving are unavailable.'
} as const;
