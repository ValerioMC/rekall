import { describe, expect, it } from 'vitest'
import { isTypingTarget, neighboursOf } from '@/features/steps/lib/step-navigation'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

function step(id: string): TaskStep {
  return { id: id as TaskStepId } as TaskStep
}

describe('neighboursOf', () => {
  const shown = [step('a'), step('b'), step('c')]

  it('names the step before and the step after the selected one', () => {
    const { previous, next } = neighboursOf(shown, 'b' as TaskStepId)

    expect(previous?.id).toBe('a')
    expect(next?.id).toBe('c')
  })

  it('has no step before the first or after the last', () => {
    expect(neighboursOf(shown, 'a' as TaskStepId).previous).toBeNull()
    expect(neighboursOf(shown, 'c' as TaskStepId).next).toBeNull()
  })

  it('offers the first step as next when the selected one is not in the list as shown', () => {
    const { previous, next } = neighboursOf(shown, 'hidden' as TaskStepId)

    expect(previous).toBeNull()
    expect(next?.id).toBe('a')
  })
})

describe('isTypingTarget', () => {
  it('is true on a field and on anything inside an editor, and false on a plain element', () => {
    const field = document.createElement('input')
    const editor = document.createElement('div')
    editor.className = 'cm-editor'
    const inside = document.createElement('span')
    editor.append(inside)

    expect(isTypingTarget(field)).toBe(true)
    expect(isTypingTarget(inside)).toBe(true)
    expect(isTypingTarget(document.createElement('button'))).toBe(false)
    expect(isTypingTarget(null)).toBe(false)
  })
})
