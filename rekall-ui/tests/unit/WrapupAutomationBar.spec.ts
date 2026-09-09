import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WrapupAutomationBar from '@/components/console/WrapupAutomationBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * The standing "generate the wrapup" directive, set once on a task. It renders under the header
 * of the description pane and the steps pane, so it is reachable from whichever surface the
 * session is driven from. The toggle saves on the spot; the directive it reveals autosaves on
 * a pause. When the toggle is off it is a single checkbox; nothing else.
 */
const rekall = 'p1' as ProjectId
const taskId = 't1' as TaskId

function makeTask(over: Partial<Task> = {}): Task {
  return {
    id: taskId,
    label: 'application-improvements',
    title: 'Application improvements',
    status: 'IN_PROGRESS',
    description: null,
    autoWrapup: false,
    wrapupDirective: null,
    projectId: rekall,
    projectLabel: 'rekall',
    projectTitle: 'Rekall',
    companyName: 'vforge',
    projectRepoFolder: null,
    documentCount: 0,
    stepCount: 3,
    stepsDone: 1,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: 'OPEN',
    reviewActive: false,
    claimedAt: null,
    acceptedAt: null,
    reviewNote: null,
    anchor: 'project:rekall task:application-improvements',
    updatedAt: '2026-09-09T10:00:00Z',
    ...over
  }
}

function seed(over: Partial<Task> = {}) {
  const store = useConsoleStore()
  store.tasks = [makeTask(over)]
  store.selectedTaskId = taskId
  store.isLoading = false
  store.saveTaskWrapup = vi.fn().mockResolvedValue(undefined)
  return store
}

let pinia: Pinia

function render() {
  return mount(WrapupAutomationBar, { global: { plugins: [pinia] } })
}

describe('WrapupAutomationBar', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    vi.useRealTimers()
  })

  it('renders nothing until a task is selected', async () => {
    const store = useConsoleStore()
    store.tasks = []
    store.selectedTaskId = null
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="wrapup-automation"]').exists()).toBe(false)
  })

  it('shows the toggle with the directive field hidden while it is off', async () => {
    seed()
    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="wrapup-auto-toggle"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="wrapup-directive-field"]').exists()).toBe(false)
  })

  it('saves the toggle the moment it is checked', async () => {
    const store = seed()
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[data-testid="wrapup-auto-toggle"] input').setValue(true)

    expect(store.saveTaskWrapup).toHaveBeenCalledWith(taskId, true, '')
  })

  it('shows the stored directive in the field when the task already carries one', async () => {
    seed({ autoWrapup: true, wrapupDirective: 'export module only' })
    const wrapper = render()
    await flushPromises()

    const field = wrapper.find('[data-testid="wrapup-directive-field"]')
    expect(field.exists()).toBe(true)
    expect(field.find('input').element.value).toBe('export module only')
  })

  it('reveals only the Directive label and its field, nothing else', async () => {
    seed({ autoWrapup: true })
    const wrapper = render()
    await flushPromises()

    const bar = wrapper.get('[data-testid="wrapup-automation"]')
    expect(bar.findAll('button')).toHaveLength(0)
    expect(bar.findAll('input')).toHaveLength(2)
    expect(bar.get('[data-testid="wrapup-directive-field"]').text()).toBe('Directive')
  })

  it('debounces the directive edit into a single save', async () => {
    vi.useFakeTimers()
    const store = seed({ autoWrapup: true })
    const wrapper = render()
    await flushPromises()

    const input = wrapper.get('[data-testid="wrapup-directive-input"]')
    await input.setValue('narrow to the parser')
    expect(store.saveTaskWrapup).not.toHaveBeenCalled()

    vi.advanceTimersByTime(700)
    await flushPromises()

    expect(store.saveTaskWrapup).toHaveBeenCalledWith(taskId, true, 'narrow to the parser')
  })
})
