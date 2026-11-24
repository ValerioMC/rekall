import { describe, expect, it, vi } from 'vitest'
import { useStepDrag, type StepDropHandlers } from '@/features/steps/composables/useStepDrag'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, state: TaskStepState = 'OPEN'): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, state } as TaskStep
}

function handlers(): StepDropHandlers {
  return { onEdge: vi.fn(), onSlot: vi.fn(), onSessionSlot: vi.fn() }
}

describe('useStepDrag', () => {
  it('takes a dragged step only next to another open step', () => {
    const drag = useStepDrag(handlers())
    const held = step('a')
    drag.start(held)

    expect(drag.accepts(step('b'))).toBe(true)
    expect(drag.accepts(held)).toBe(false)
    expect(drag.accepts(step('c', 'CLAIMED'))).toBe(false)
  })

  it('takes nothing while no step is held', () => {
    expect(useStepDrag(handlers()).accepts(step('b'))).toBe(false)
  })

  it('names the edge the pointer is on, for the step it is over only', () => {
    const drag = useStepDrag(handlers())
    drag.start(step('a'))
    const target = step('b')

    drag.hover(target, true)

    expect(drag.edgeOf(target)).toBe('after')
    expect(drag.edgeOf(step('c'))).toBeNull()
  })

  it('reports the move on a drop, then forgets the drag', () => {
    const { onEdge, ...rest } = handlers()
    const drag = useStepDrag({ onEdge, ...rest })
    const moving = step('a')
    const target = step('b')
    drag.start(moving)
    drag.hover(target, false)

    drag.drop(target)

    expect(onEdge).toHaveBeenCalledWith(moving, target, false)
    expect(drag.dragged.value).toBeNull()
    expect(drag.edgeOf(target)).toBeNull()
  })

  it('moves nothing when the drag ends away from a step', () => {
    const given = handlers()
    const drag = useStepDrag(given)
    drag.start(step('a'))

    drag.end()

    expect(given.onEdge).not.toHaveBeenCalled()
    expect(drag.dragged.value).toBeNull()
  })

  it('lights one slot at a time, and the edge of a step puts the slot out', () => {
    const drag = useStepDrag(handlers())
    const before = step('b')
    drag.start(step('a'))

    drag.hoverSlot(before)
    expect(drag.slotActive(before)).toBe(true)
    expect(drag.slotActive(null)).toBe(false)

    drag.hover(step('c'), true)
    expect(drag.slotActive(before)).toBe(false)
  })

  it('reports a held step dropped on a slot, with the step the slot is in front of', () => {
    const given = handlers()
    const drag = useStepDrag(given)
    const moving = step('a')
    const before = step('b')
    drag.start(moving)

    drag.dropOnSlot(before)

    expect(given.onSlot).toHaveBeenCalledWith(moving, before)
    expect(drag.holding.value).toBe(false)
  })

  it('reports a held session dropped on the slot at the end, and takes no step edge while it is held', () => {
    const given = handlers()
    const drag = useStepDrag(given)
    const session: StepSession = { number: 2, steps: [step('a')] }
    drag.startSession(session)

    expect(drag.accepts(step('b'))).toBe(false)
    drag.dropOnSlot(null)

    expect(given.onSessionSlot).toHaveBeenCalledWith(session, null)
    expect(given.onSlot).not.toHaveBeenCalled()
  })
})
