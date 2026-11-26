import { describe, expect, it } from 'vitest'
import { chatStatus, composerState } from '@/features/chat/lib/chat-status'
import { describePermission } from '@/features/chat/lib/describe-permission'
import { matchingCommands } from '@/features/chat/lib/slash-commands'
import { sessionStep } from '@/features/chat/lib/step-chip'
import { emptyTimeline } from '@/features/chat/lib/timeline'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'
import { chatSession } from './support'

function steps(...startsSession: boolean[]): TaskStep[] {
  return startsSession.map((starts, index) => ({ id: `s${index + 1}` as TaskStepId, startsSession: starts }) as TaskStep)
}

describe('chatStatus', () => {
  const connected = { connected: true, reconnecting: false }

  it('ranks an answer owed to the person above everything else', () => {
    const waiting = { ...emptyTimeline(), turn: 'waiting' as const, pending: { requestId: 'r', toolUseId: 'u', name: 'Bash', input: { tool: 'other', name: 'x', text: '' } as const } }

    expect(chatStatus({ timeline: waiting, session: chatSession(), connected: false, reconnecting: true })).toBe('waiting')
    expect(chatStatus({ timeline: emptyTimeline(), session: chatSession({ awaitingPermission: true }), ...connected })).toBe('waiting')
  })

  it('reads ended, reconnecting, connecting, working and idle in that order', () => {
    expect(chatStatus({ timeline: { ...emptyTimeline(), ended: true }, session: chatSession(), ...connected })).toBe('ended')
    expect(chatStatus({ timeline: emptyTimeline(), session: chatSession({ live: false }), ...connected })).toBe('ended')
    expect(chatStatus({ timeline: emptyTimeline(), session: chatSession(), connected: false, reconnecting: true })).toBe('reconnecting')
    expect(chatStatus({ timeline: emptyTimeline(), session: chatSession(), connected: false, reconnecting: false })).toBe('connecting')
    expect(chatStatus({ timeline: { ...emptyTimeline(), turn: 'working' }, session: chatSession(), ...connected })).toBe('working')
    expect(chatStatus({ timeline: emptyTimeline(), session: chatSession(), ...connected })).toBe('idle')
  })

  it('pauses the composer while waiting and leaves the other states as they are', () => {
    expect(composerState('waiting')).toBe('paused')
    expect(composerState('working')).toBe('working')
  })
})

describe('describePermission', () => {
  it('names what is asked in plain words', () => {
    expect(describePermission({ tool: 'bash', command: 'ls', description: null })).toBe('Claude wants to run a shell command')
    expect(describePermission({ tool: 'edit', filePath: 'a.rs', oldString: '', newString: '', replaceAll: false })).toBe('Claude wants to edit a.rs')
    expect(describePermission({ tool: 'write', filePath: 'a.rs', content: 'one\ntwo\n' })).toBe('Claude wants to write a.rs (2 lines)')
    expect(describePermission({ tool: 'read', filePath: 'a.rs' })).toBe('Claude wants to read a.rs')
    expect(describePermission({ tool: 'search', name: 'Grep', pattern: 'todo', path: 'src' })).toBe('Claude wants to search for todo in src')
    expect(describePermission({ tool: 'search', name: 'Grep', pattern: 'todo', path: null })).toBe('Claude wants to search for todo')
    expect(describePermission({ tool: 'other', name: 'WebFetch', text: '' })).toBe('Claude wants to use WebFetch')
  })
})

describe('matchingCommands', () => {
  it('offers commands only for a leading slash with no space yet', () => {
    expect(matchingCommands('hello')).toEqual([])
    expect(matchingCommands('/model opus')).toEqual([])
    expect(matchingCommands('/c').map((command) => command.name)).toEqual(['/context', '/compact'])
  })
})

describe('sessionStep', () => {
  it('names the step by its number and title, with the steps its session covers', () => {
    const checklist = steps(false, false, true, false, false)

    expect(sessionStep('s4' as TaskStepId, checklist)).toEqual({
      name: `Step 4 · ${checklist[3]!.title}`,
      coverage: 'session covers steps 3 to 5'
    })
    expect(sessionStep('s1' as TaskStepId, steps(false, true))?.coverage).toBeNull()
  })

  it('names nothing for a session on no step or on a step it cannot find', () => {
    expect(sessionStep(null, steps(false))).toBeNull()
    expect(sessionStep('s9' as TaskStepId, steps(false))).toBeNull()
  })
})
