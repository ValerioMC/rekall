import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ContextSizeChip from '@/components/console/ContextSizeChip.vue'
import type { ContextSize } from '@/model/context'
import type { TaskId } from '@/model/branded'

const fetchContextSize = vi.fn<(taskId: TaskId) => Promise<ContextSize>>()
vi.mock('@/api/context.api', () => ({ fetchContextSize: (taskId: TaskId) => fetchContextSize(taskId) }))

const measured: ContextSize = {
  characters: 43_400,
  estimatedTokens: 12_400,
  parts: [
    { label: 'Note: kmaster14.md', characters: 35_000, reference: false },
    { label: 'Wrapup', characters: 7_000, reference: false },
    { label: 'Note: onboarding.md', characters: 400, reference: true }
  ]
}

describe('the context size chip', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    fetchContextSize.mockReset().mockResolvedValue(measured)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('measures once things settle, and shows the estimate', async () => {
    const wrapper = mount(ContextSizeChip, { props: { taskId: 't1' as TaskId } })
    expect(wrapper.text()).toContain('measuring')

    await vi.advanceTimersByTimeAsync(450)
    await flushPromises()

    expect(fetchContextSize).toHaveBeenCalledOnce()
    expect(wrapper.get('[data-testid="context-size"]').text()).toBe('≈ 12.4k tokens')
  })

  it('lists the parts, marks the reference notes, and suggests sending a heavy note by reference', async () => {
    const wrapper = mount(ContextSizeChip, { props: { taskId: 't1' as TaskId }, attachTo: document.body })
    await vi.advanceTimersByTimeAsync(450)
    await flushPromises()

    await wrapper.get('[data-testid="context-size"]').trigger('click')
    const parts = document.body.querySelector('[data-testid="context-size-parts"]')

    expect(parts?.textContent).toContain('Note: kmaster14.md')
    expect(parts?.textContent).toContain('reference')
    expect(parts?.textContent).toContain('By reference')
    wrapper.unmount()
  })

  it('says the size is unknown when the server cannot measure it', async () => {
    fetchContextSize.mockRejectedValue(new Error('down'))
    const wrapper = mount(ContextSizeChip, { props: { taskId: 't1' as TaskId } })
    await vi.advanceTimersByTimeAsync(450)
    await flushPromises()

    expect(wrapper.text()).toContain('size unknown')
  })
})
