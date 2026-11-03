import { describe, expect, it } from 'vitest'
import { taskSessionState } from '@/features/claude/model/task-session'
import type { TaskId } from '@/model/branded'
import { chatSession } from '../chat/support'

const TASK = 't-1' as TaskId

describe('taskSessionState', () => {
  it('is none for a task no session was opened on, whatever runs on the others', () => {
    expect(taskSessionState(TASK, [])).toBe('NONE')
    expect(taskSessionState(TASK, [chatSession({ taskId: 't-2' as TaskId })])).toBe('NONE')
  })

  it('is live while a session is open on the task, terminal or chat', () => {
    expect(taskSessionState(TASK, [chatSession()])).toBe('LIVE')
    expect(taskSessionState(TASK, [chatSession({ kind: 'TERMINAL' })])).toBe('LIVE')
  })

  it('is ended once every session on the task has ended', () => {
    expect(taskSessionState(TASK, [chatSession({ live: false })])).toBe('ENDED')
  })

  it('is live when one ended and a newer one is open', () => {
    const sessions = [chatSession({ live: false }), chatSession({ id: 'chat-2' as never })]

    expect(taskSessionState(TASK, sessions)).toBe('LIVE')
  })

  it('asks for approval above everything else while a live session waits for it', () => {
    const sessions = [chatSession({ awaitingPermission: true }), chatSession({ id: 'chat-2' as never })]

    expect(taskSessionState(TASK, sessions)).toBe('APPROVAL')
    expect(taskSessionState(TASK, [chatSession({ live: false, awaitingPermission: true })])).toBe('ENDED')
  })
})
