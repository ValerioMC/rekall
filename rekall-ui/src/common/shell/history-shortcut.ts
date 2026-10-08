export type HistoryDirection = 'back' | 'forward'

type KeyChord = Pick<KeyboardEvent, 'code' | 'metaKey' | 'altKey' | 'ctrlKey' | 'shiftKey'>

const DIRECTION_BY_CODE: Readonly<Record<string, HistoryDirection>> = {
  ArrowLeft: 'back',
  ArrowRight: 'forward'
}

export function runsOnMac(platform: string): boolean {
  return /mac/i.test(platform)
}

/**
 * Cmd+Option+Arrow on a Mac, Alt+Arrow elsewhere (the browsers' own chord). Any other modifier
 * on top, Shift included, leaves the key to its text-editing meaning.
 */
export function historyDirectionOf(chord: KeyChord, mac: boolean): HistoryDirection | null {
  const direction = DIRECTION_BY_CODE[chord.code]
  if (!direction || chord.shiftKey || chord.ctrlKey) return null
  return chord.altKey && chord.metaKey === mac ? direction : null
}

export function historyShortcutLabel(direction: HistoryDirection, mac: boolean): string {
  const arrow = direction === 'back' ? '←' : '→'
  return mac ? `⌘⌥${arrow}` : `Alt+${arrow}`
}
