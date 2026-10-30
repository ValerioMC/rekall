import type { Terminal } from '@/features/claude/model/terminal'
import type { Timeline } from '../model/timeline'

export type ChatStatus = 'connecting' | 'working' | 'idle' | 'waiting' | 'reconnecting' | 'ended'

export type ComposerState = 'idle' | 'working' | 'paused' | 'connecting' | 'reconnecting' | 'ended'

export interface StatusReading {
  readonly timeline: Timeline
  readonly session: Terminal | null
  readonly connected: boolean
  readonly reconnecting: boolean
}

/** What the window says the session is doing: an answer owed to the person outranks every other state. */
export function chatStatus(reading: StatusReading): ChatStatus {
  const { timeline, session, connected, reconnecting } = reading
  if (timeline.ended || (session !== null && !session.live)) return 'ended'
  if (timeline.pending || session?.awaitingPermission) return 'waiting'
  if (reconnecting) return 'reconnecting'
  if (!connected) return 'connecting'
  return timeline.turn === 'working' ? 'working' : 'idle'
}

export function composerState(status: ChatStatus): ComposerState {
  return status === 'waiting' ? 'paused' : status
}
