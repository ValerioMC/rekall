import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import RevisionRunBar from '@/features/feedback/components/RevisionRunBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Feedback } from '@/features/feedback/model/feedback'
import { entry, TASK } from './fixtures'

vi.mock('@/features/feedback/api/feedback.api', () => ({ fetchFeedback: vi.fn(), retargetFeedback: vi.fn(), discardRevision: vi.fn(), startPass: vi.fn() }))

function bar(thread: Feedback[], revisionsShown = false) {
  const store = useConsoleStore()
  store.selectedTaskId = TASK
  store.feedbackByTask = { [TASK]: thread }
  const retarget = vi.spyOn(store, 'retargetFeedback').mockResolvedValue()
  const wrapper = mount(RevisionRunBar, { props: { taskId: TASK, anchor: 'project:vega task:report-builder', folder: '/code/vega', revisionsShown } })
  return { wrapper, retarget }
}

describe('the bar that runs a waiting revision', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('counts the revisions waiting and holds the one Run revision button', () => {
    const { wrapper } = bar([entry(1), entry(2)])

    expect(wrapper.get('[data-testid="revision-run-summary"]').text()).toBe('2 revisions waiting')
    expect(wrapper.findAll('[data-testid="apply-feedback"]')).toHaveLength(1)
  })

  it('shows what the waiting revision is applied to', () => {
    expect(bar([entry(1)]).wrapper.get('[data-testid="revision-target-IMPLEMENTATION"]').attributes('aria-pressed')).toBe('true')
    expect(bar([entry(1, { kind: 'PLAN' })]).wrapper.get('[data-testid="revision-target-PLAN"]').attributes('aria-pressed')).toBe('true')
  })

  it('points the waiting revision at the other target when it is chosen', async () => {
    const { wrapper, retarget } = bar([entry(1), entry(2, { state: 'APPLIED' })])

    await wrapper.get('[data-testid="revision-target-PLAN"]').trigger('click')
    await flushPromises()

    expect(retarget).toHaveBeenCalledTimes(1)
    expect(retarget).toHaveBeenCalledWith(entry(1), 'PLAN')
  })

  it('offers to see the changes unless they are already on screen', async () => {
    const closed = bar([entry(1)]).wrapper
    await closed.get('[data-testid="revision-run-view"]').trigger('click')
    expect(closed.emitted('showRevisions')).toHaveLength(1)

    expect(bar([entry(1)], true).wrapper.find('[data-testid="revision-run-view"]').exists()).toBe(false)
  })
})
