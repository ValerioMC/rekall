import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

/**
 * What the sessions opened on a task say about it, for the row that lists the task.
 *
 *   APPROVAL  a chat session waits for an answer from you: it outranks every other state.
 *   LIVE      a session is open on the task.
 *   ENDED     a session ran on the task and has ended.
 *   NONE      no session was ever opened on it.
 */
export type TaskSessionState = 'APPROVAL' | 'LIVE' | 'ENDED' | 'NONE'

export const TASK_SESSION_LABEL: Readonly<Record<TaskSessionState, string>> = {
  APPROVAL: 'A session is waiting for your approval',
  LIVE: 'A session is open on it',
  ENDED: 'A session has run on it and ended',
  NONE: ''
}

export function taskSessionState(taskId: TaskId, sessions: readonly Terminal[]): TaskSessionState {
  const own = sessions.filter((session) => session.taskId === taskId)
  if (own.some((session) => session.live && session.awaitingPermission)) return 'APPROVAL'
  if (own.some((session) => session.live)) return 'LIVE'
  return own.length > 0 ? 'ENDED' : 'NONE'
}
