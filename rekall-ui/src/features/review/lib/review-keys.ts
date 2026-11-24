export type ReviewKeyAction = 'next' | 'previous' | 'accept' | 'send-back'

const ACTIONS: Readonly<Record<string, ReviewKeyAction>> = { j: 'next', k: 'previous', a: 'accept', s: 'send-back' }

/** The decision a bare key stands for on the Review screen: J and K move, A accepts, S sends back. */
export function reviewKeyAction(event: Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'repeat'>): ReviewKeyAction | null {
  if (event.metaKey || event.ctrlKey || event.altKey || event.repeat) return null
  return ACTIONS[event.key.toLowerCase()] ?? null
}
