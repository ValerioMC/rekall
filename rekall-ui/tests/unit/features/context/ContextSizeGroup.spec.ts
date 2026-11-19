import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ContextSizeGroup from '@/features/context/components/ContextSizeGroup.vue'
import type { ContextSize } from '@/features/context/model/context'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

const fetchContextSize = vi.fn<(taskId: TaskId) => Promise<ContextSize>>()
vi.mock('@/features/context/api/context.api', () => ({ fetchContextSize: (taskId: TaskId) => fetchContextSize(taskId) }))

const measured: ContextSize = {
  characters: 43_400,
  estimatedTokens: 12_400,
  parts: [
    { label: 'Note: kmaster14.md', characters: 35_000, reference: false },
    { label: 'Wrapup', characters: 7_000, reference: false },
    { label: 'Note: onboarding.md', characters: 400, reference: true }
  ]
}

const task = { id: 't1' as TaskId, projectId: 'p1' } as Task

async function settle(): Promise<void> {
  await vi.advanceTimersByTimeAsync(450)
  await flushPromises()
}

describe('the context group of the properties bar', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    fetchContextSize.mockReset().mockResolvedValue(measured)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('says it is measuring, then gives the estimate with one row per part, the heaviest first', async () => {
    const wrapper = mount(ContextSizeGroup, { props: { task } })
    expect(wrapper.get('[data-testid="context-size"]').text()).toBe('measuring…')

    await settle()

    expect(wrapper.get('[data-testid="context-size"]').text()).toBe('≈ 12.4k tokens')
    const rows = wrapper.findAll('[data-testid="context-size-parts"] li')
    expect(rows.map((row) => row.text())).toEqual([
      expect.stringContaining('Note: kmaster14.md'),
      expect.stringContaining('Wrapup'),
      expect.stringContaining('Note: onboarding.md')
    ])
    expect(rows[2]!.text()).toContain('reference')
  })

  it('suggests sending a heavy note by reference', async () => {
    const wrapper = mount(ContextSizeGroup, { props: { task } })
    await settle()

    expect(wrapper.text()).toContain('By reference')
  })

  it('says the size is unknown when the server cannot measure it', async () => {
    fetchContextSize.mockRejectedValue(new Error('down'))
    const wrapper = mount(ContextSizeGroup, { props: { task } })
    await settle()

    expect(wrapper.get('[data-testid="context-size"]').text()).toBe('size unknown')
  })
})
