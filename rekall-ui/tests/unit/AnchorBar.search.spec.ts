import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises } from '@vue/test-utils'
import AnchorBar from '@/components/console/AnchorBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from './support/mountWithHeaderChrome'
import type { SearchHit } from '@/model/search'
import type { TaskId, TaskStepId } from '@/model/branded'

const searchText = vi.fn<(term: string) => Promise<SearchHit[]>>()
vi.mock('@/api/search.api', () => ({ searchText: (term: string) => searchText(term) }))

const hits: SearchHit[] = [
  {
    kind: 'DESCRIPTION',
    taskId: 't1' as TaskId,
    stepId: null,
    documentId: null,
    title: 'Settlement',
    where: 'project:vega task:settlement',
    excerpt: 'The nightly settlement batch reconciles the ledger.'
  },
  {
    kind: 'STEP',
    taskId: 't1' as TaskId,
    stepId: 's1' as TaskStepId,
    documentId: null,
    title: 'Wire the batch',
    where: 'project:vega task:settlement',
    excerpt: 'Schedule the settlement batch at 02:00.'
  }
]

function render() {
  return mountWithHeaderChrome(AnchorBar, {
    global: { stubs: { ClaudeUsageMeter: true, ReviewQueueButton: true } }
  })
}

describe('the anchor bar, searching the text', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    searchText.mockReset().mockResolvedValue(hits)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('searches a phrase once typing pauses, and lists where it was found', async () => {
    const wrapper = render()
    const input = wrapper.find('[data-testid="anchor-input"]')
    await input.trigger('focus')
    await input.setValue('settlement batch')

    expect(searchText).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(250)
    await flushPromises()

    expect(searchText).toHaveBeenCalledOnce()
    expect(searchText).toHaveBeenCalledWith('settlement batch')
    expect(wrapper.findAll('[data-testid="search-hit"]')).toHaveLength(2)
    wrapper.unmount()
  })

  it('never searches an anchor, which the filter already resolves', async () => {
    const wrapper = render()
    const input = wrapper.find('[data-testid="anchor-input"]')
    await input.trigger('focus')
    await input.setValue('project:vega task:settlement')
    await vi.advanceTimersByTimeAsync(250)

    expect(searchText).not.toHaveBeenCalled()
    expect(wrapper.find('[data-testid="search-hits"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('opens the hit chosen with the arrows, and clears the bar behind it', async () => {
    const store = useConsoleStore()
    store.openSearchHit = vi.fn()
    const wrapper = render()
    const input = wrapper.find('[data-testid="anchor-input"]')
    await input.trigger('focus')
    await input.setValue('settlement batch')
    await vi.advanceTimersByTimeAsync(250)
    await flushPromises()

    await input.trigger('keydown', { key: 'ArrowDown' })
    await input.trigger('keydown', { key: 'ArrowDown' })
    await input.trigger('keydown', { key: 'Enter' })

    expect(store.openSearchHit).toHaveBeenCalledWith(hits[1])
    expect(store.filter).toBe('')
    wrapper.unmount()
  })
})
