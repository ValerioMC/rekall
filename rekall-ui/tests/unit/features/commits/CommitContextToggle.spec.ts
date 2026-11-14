import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import CommitContextToggle from '@/features/commits/components/CommitContextToggle.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { TaskId } from '@/model/branded'

function reference(inContext: boolean): CommitReference {
  return {
    id: 'c1',
    taskId: 't1' as TaskId,
    stepId: null,
    stepTitle: null,
    commitHash: 'abc1234def',
    comment: 'Wire up the button',
    inContext,
    createdAt: '2026-09-14T10:00:00Z'
  }
}

describe('CommitContextToggle', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('names the action in words on the labelled variant', () => {
    const off = mount(CommitContextToggle, { props: { reference: reference(false), variant: 'labelled' } })
    const on = mount(CommitContextToggle, { props: { reference: reference(true), variant: 'labelled' } })

    expect(off.text()).toBe('Add to /rk context')
    expect(off.attributes('aria-pressed')).toBe('false')
    expect(on.text()).toBe('In the /rk context')
    expect(on.attributes('aria-pressed')).toBe('true')
  })

  it('flips the choice through the store on the labelled variant', async () => {
    const store = useConsoleStore()
    store.setCommitReferenceInContext = vi.fn().mockResolvedValue(undefined)
    const wrapper = mount(CommitContextToggle, { props: { reference: reference(true), variant: 'labelled' } })

    await wrapper.trigger('click')
    await flushPromises()

    expect(store.setCommitReferenceInContext).toHaveBeenCalledWith('c1', false)
  })
})
