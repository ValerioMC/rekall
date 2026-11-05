import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useFeedbackComposer } from '@/features/feedback/composables/useFeedbackComposer'
import type { TaskStep } from '@/features/catalog/model/catalog'
import { TASK } from './fixtures'

function stepOf(task: string): TaskStep {
  return { id: 's-1', taskId: task, position: 0, state: 'OPEN', title: 'Build' } as unknown as TaskStep
}

describe('useFeedbackComposer', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('defaults to feedback on the work for a task with no steps', () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK

    expect(useFeedbackComposer().kind.value).toBe('IMPLEMENTATION')
  })

  it('defaults to feedback on the plan for a task with steps, and keeps the kind the person chose', () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    store.steps = [stepOf(TASK)]
    const composer = useFeedbackComposer()

    expect(composer.kind.value).toBe('PLAN')
    composer.chooseKind('IMPLEMENTATION')
    expect(composer.kind.value).toBe('IMPLEMENTATION')
  })

  it('submits the draft as feedback on the task in view, then leaves nothing in the composer', async () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    const add = vi.spyOn(store, 'addFeedback').mockResolvedValue()
    const composer = useFeedbackComposer()
    composer.draft.value = 'The totals are wrong'

    await composer.submit()

    expect(add).toHaveBeenCalledWith(TASK, 'IMPLEMENTATION', 'The totals are wrong', null)
    expect(composer.draft.value).toBe('')
  })

  it('does not submit a blank draft', async () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    const add = vi.spyOn(store, 'addFeedback').mockResolvedValue()
    const composer = useFeedbackComposer()
    composer.draft.value = '   '

    await composer.submit()

    expect(composer.canSubmit.value).toBe(false)
    expect(add).not.toHaveBeenCalled()
  })

  it('submits against the applied entry it adds detail to, and forgets it afterwards', async () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    const add = vi.spyOn(store, 'addFeedback').mockResolvedValue()
    const composer = useFeedbackComposer()
    composer.followUp(3)
    composer.draft.value = 'Still wrong for refunds'

    await composer.submit()

    expect(add).toHaveBeenCalledWith(TASK, 'IMPLEMENTATION', 'Still wrong for refunds', 3)
    expect(composer.followsSeq.value).toBeNull()
  })

  it('lets the person drop the entry it was aimed at', () => {
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    const composer = useFeedbackComposer()
    composer.followUp(2)

    composer.dropFollowUp()

    expect(composer.followsSeq.value).toBeNull()
  })
})
