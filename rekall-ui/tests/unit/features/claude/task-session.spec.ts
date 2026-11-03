import { describe, expect, it } from 'vitest'
import { taskSessionState } from '@/features/claude/model/task-session'
import type { TaskId } from '@/model/branded'
import { chatSession } from '../chat/support'

const TASK = 't-1' as TaskId

describe('taskSessionState', () => {
  it('is none for a task no session was opened on, whatever runs on the others', () => {
    expect(taskSessionState(TASK, [], false)).toBe('NONE')
    expect(taskSessionState(TASK, [chatSession({ taskId: 't-2' as TaskId })], false)).toBe('NONE')
  })

  it('is live while a session is open on the task, terminal or chat', () => {
    expect(taskSessionState(TASK, [chatSession()], false)).toBe('LIVE')
    expect(taskSessionState(TASK, [chatSession({ kind: 'TERMINAL' })], false)).toBe('LIVE')
  })

  it('is ended once every session on the task has ended', () => {
    expect(taskSessionState(TASK, [chatSession({ live: false })], false)).toBe('ENDED')
  })

  it('is live when one ended and a newer one is open', () => {
    const sessions = [chatSession({ live: false }), chatSession({ id: 'chat-2' as never })]

    expect(taskSessionState(TASK, sessions, false)).toBe('LIVE')
  })

  it('asks for approval above everything else while a live session waits for it', () => {
    const sessions = [chatSession({ awaitingPermission: true }), chatSession({ id: 'chat-2' as never })]

    expect(taskSessionState(TASK, sessions, false)).toBe('APPROVAL')
    expect(taskSessionState(TASK, [chatSession({ live: false, awaitingPermission: true })], false)).toBe('ENDED')
  })

  it('is claimed once work is handed in and a session ran on the task, live or ended', () => {
    expect(taskSessionState(TASK, [chatSession()], true)).toBe('CLAIMED')
    expect(taskSessionState(TASK, [chatSession({ live: false })], true)).toBe('CLAIMED')
  })

  it('stays none for a claimed task no session was opened on, and asks for approval before claimed', () => {
    expect(taskSessionState(TASK, [], true)).toBe('NONE')
    expect(taskSessionState(TASK, [chatSession({ awaitingPermission: true })], true)).toBe('APPROVAL')
  })
})
