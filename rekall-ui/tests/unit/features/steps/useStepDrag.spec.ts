import { describe, expect, it, vi } from 'vitest'
import { useStepDrag } from '@/features/steps/composables/useStepDrag'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, state: TaskStepState = 'OPEN'): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, state } as TaskStep
}

describe('useStepDrag', () => {
  it('takes a dragged step only next to another open step', () => {
    const drag = useStepDrag(vi.fn())
    const held = step('a')
    drag.start(held)

    expect(drag.accepts(step('b'))).toBe(true)
    expect(drag.accepts(held)).toBe(false)
    expect(drag.accepts(step('c', 'CLAIMED'))).toBe(false)
  })

  it('takes nothing while no step is held', () => {
    expect(useStepDrag(vi.fn()).accepts(step('b'))).toBe(false)
  })

  it('names the edge the pointer is on, for the step it is over only', () => {
    const drag = useStepDrag(vi.fn())
    drag.start(step('a'))
    const target = step('b')

    drag.hover(target, true)

    expect(drag.edgeOf(target)).toBe('after')
    expect(drag.edgeOf(step('c'))).toBeNull()
  })

  it('reports the move on a drop, then forgets the drag', () => {
    const onDrop = vi.fn()
    const drag = useStepDrag(onDrop)
    const moving = step('a')
    const target = step('b')
    drag.start(moving)
    drag.hover(target, false)

    drag.drop(target)

    expect(onDrop).toHaveBeenCalledWith(moving, target, false)
    expect(drag.dragged.value).toBeNull()
    expect(drag.edgeOf(target)).toBeNull()
  })

  it('moves nothing when the drag ends away from a step', () => {
    const onDrop = vi.fn()
    const drag = useStepDrag(onDrop)
    drag.start(step('a'))

    drag.end()

    expect(onDrop).not.toHaveBeenCalled()
    expect(drag.dragged.value).toBeNull()
  })
})
