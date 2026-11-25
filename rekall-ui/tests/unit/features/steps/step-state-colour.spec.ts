import { describe, expect, it } from 'vitest'
import { STEP_STATE_DOT, standingTextClass } from '@/features/steps/model/step-state-colour'
import { TASK_STEP_STATES } from '@/features/steps/model/task-step'

describe('step state colours', () => {
  it('gives every state a dot colour and no two states the same one', () => {
    const colours = TASK_STEP_STATES.map((state) => STEP_STATE_DOT[state])

    expect(colours.every((colour) => colour.startsWith('bg-'))).toBe(true)
    expect(new Set(colours).size).toBe(TASK_STEP_STATES.length)
  })

  it('reads a claimed step in the accent and a running one in the session colour', () => {
    expect(standingTextClass('CLAIMED')).toContain('text-accent')
    expect(standingTextClass('RUNNING')).toContain('text-session-strong')
    expect(standingTextClass('CLAIMED')).not.toBe(standingTextClass('RUNNING'))
  })

  it('keeps the other states quiet', () => {
    expect(['DRAFT', 'OPEN', 'DONE'].map((state) => standingTextClass(state as never))).toEqual([
      'text-text-subtle',
      'text-text-subtle',
      'text-text-subtle'
    ])
  })
})
