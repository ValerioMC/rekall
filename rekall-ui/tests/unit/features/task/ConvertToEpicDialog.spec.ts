import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ConvertToEpicDialog from '@/features/task/components/ConvertToEpicDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { EpicConversion } from '@/features/catalog/model/epic-conversion'
import type { TaskId } from '@/model/branded'
import { taskFixture } from '../../support/taskFixture'

const loaded = vi.hoisted(() => ({ conversion: null as EpicConversion | null }))

vi.mock('@/features/catalog/api/epic-conversion.api', () => ({
  fetchEpicConversion: vi.fn(async () => loaded.conversion)
}))

const id = 't1' as TaskId

function conversion(over: Partial<EpicConversion> = {}): EpicConversion {
  return {
    taskId: id,
    steps: 0,
    stepsDone: 0,
    routes: 0,
    hasWrapup: false,
    commits: 0,
    deviations: 0,
    waitsFor: 0,
    neededBy: 0,
    skills: 0,
    hasPullRequest: false,
    epicCode: null,
    epicTitle: null,
    queued: false,
    ...over
  }
}

function render() {
  const pinia = createPinia()
  setActivePinia(pinia)
  const store = useConsoleStore()
  store.convertTaskToEpic = vi.fn().mockResolvedValue(undefined)
  const wrapper = mount(ConvertToEpicDialog, {
    attachTo: document.body,
    props: { task: { ...taskFixture(id), title: 'Session handoff', code: 'VG-210', documentCount: 3 } },
    global: { plugins: [pinia] }
  })
  return { wrapper, store }
}

describe('ConvertToEpicDialog', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('warns with each part that is deleted and converts when confirmed', async () => {
    loaded.conversion = conversion({ steps: 12, stepsDone: 5, commits: 3 })
    const { wrapper, store } = render()
    await flushPromises()

    expect(wrapper.findAll('[data-testid="conversion-loss"]').map((row) => row.text())).toEqual([
      expect.stringContaining('12 steps (5 done)'),
      expect.stringContaining('3 logged commits')
    ])
    expect(wrapper.get('[data-testid="convert-to-epic-confirm"]').text()).toBe('Convert and delete')

    await wrapper.get('[data-testid="convert-to-epic-confirm"]').trigger('click')
    await flushPromises()

    expect(store.convertTaskToEpic).toHaveBeenCalledWith(id)
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('asks for a plain confirmation when nothing is deleted', async () => {
    loaded.conversion = conversion()
    const { wrapper } = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="conversion-losses"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="convert-to-epic-confirm"]').text()).toBe('Convert to epic')
    wrapper.unmount()
  })

  it('cannot convert a task that is on the run queue', async () => {
    loaded.conversion = conversion({ queued: true })
    const { wrapper } = render()
    await flushPromises()

    expect(wrapper.get('[data-testid="conversion-queued"]').text()).toContain('run queue')
    expect(wrapper.get('[data-testid="convert-to-epic-confirm"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })
})
