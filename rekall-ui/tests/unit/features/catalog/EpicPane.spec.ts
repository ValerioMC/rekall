import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import EpicPane from '@/features/catalog/components/epic/EpicPane.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'
import { taskFixture } from '../../support/taskFixture'

vi.mock('@/features/context/api/context.api', () => ({
  fetchContextSize: vi.fn(async () => ({ characters: 3500, estimatedTokens: 1000, parts: [] }))
}))

vi.mock('@/features/project/api/project-components.api', () => ({ fetchProjectComponents: vi.fn(async () => []) }))

vi.mock('@/features/connection/api/connections.api', () => ({ fetchProjectConnections: vi.fn(async () => []) }))

/**
 * An epic in the console: its progress, its tasks and its description in one pane. The writes go
 * through the store, stubbed here and covered in `console.spec`.
 */
const epic = 'e1' as TaskId
const done = 't1' as TaskId
const open = 't2' as TaskId
const loose = 't3' as TaskId
const elsewhere = 't4' as TaskId
const otherEpic = 'e2' as TaskId
const otherProject = 't5' as TaskId

const task = (id: TaskId, changes: Partial<Task> = {}): Task => ({ ...taskFixture(id), title: `Task ${id}`, ...changes })

let pinia: Pinia

function seed() {
  const store = useConsoleStore()
  store.tasks = [
    task(epic, { kind: 'EPIC', title: 'Epics', description: 'The concept the tasks share.' }),
    task(done, { epicId: epic, status: 'DONE' }),
    task(open, { epicId: epic, status: 'IN_PROGRESS' }),
    task(loose),
    task(elsewhere, { epicId: otherEpic }),
    task(otherEpic, { kind: 'EPIC', title: 'Another epic' }),
    task(otherProject, { projectId: 'p2' as ProjectId })
  ]
  store.selectedTaskId = epic
  store.isLoading = false
  store.setTaskEpic = vi.fn().mockResolvedValue(undefined)
  store.saveTaskDescription = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  return mount(EpicPane, {
    attachTo: document.body,
    global: { plugins: [pinia], stubs: { AppMarkdownEditor: true, NotesButton: true, RecordDialog: true, TaskTitleRow: true, TaskLabelsStrip: true } }
  })
}

describe('EpicPane', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  afterEach(() => vi.useRealTimers())

  it('shows how far the tasks are, the tasks, and the description, with no way to run the epic', async () => {
    seed()
    const wrapper = render()
    await flushPromises()

    expect(wrapper.get('[data-testid="epic-progress-panel"]').text()).toContain('1')
    expect(wrapper.findAll('[data-testid="epic-task-row"]').map((row) => row.text())).toEqual([
      expect.stringContaining('Task t2'),
      expect.stringContaining('Task t1')
    ])
    expect(wrapper.getComponent({ name: 'AppMarkdownEditor' }).props('modelValue')).toBe('The concept the tasks share.')
    expect(wrapper.findComponent({ name: 'OpenTerminalButton' }).exists()).toBe(false)
    expect(wrapper.findComponent({ name: 'LaunchClaudeCodeButton' }).exists()).toBe(false)
    wrapper.unmount()
  })

  it('offers the tasks of its project that are not in it, and files the one picked', async () => {
    const store = seed()
    const wrapper = render()

    await wrapper.get('[data-testid="epic-attach-task"]').trigger('click')
    const options = wrapper.findAll('[data-testid="epic-task-option"]')
    expect(options.map((option) => option.text())).toEqual([
      expect.stringContaining('Task t3'),
      expect.stringContaining('in Another epic')
    ])

    await options[0]!.trigger('mousedown')
    await flushPromises()

    expect(store.setTaskEpic).toHaveBeenCalledWith(loose, epic)
    expect(wrapper.find('[data-testid="epic-task-picker"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('takes a task out of the epic at once, with an Undo that files it back', async () => {
    const store = seed()
    const wrapper = render()

    await wrapper.findAll('[data-testid="epic-remove-task"]')[0]!.trigger('click')
    await flushPromises()
    expect(store.setTaskEpic).toHaveBeenCalledWith(open, null)

    const toast = useToastStore().toasts.at(-1)!
    toast.action!.run()
    await flushPromises()
    expect(store.setTaskEpic).toHaveBeenLastCalledWith(open, epic)
    wrapper.unmount()
  })

  it('creates a new task already filed under the epic', async () => {
    seed()
    const wrapper = render()

    await wrapper.get('[data-testid="epic-create-task"]').trigger('click')

    expect(wrapper.getComponent(RecordDialog).props('draft')).toMatchObject({ kind: 'task', epicId: epic })
    wrapper.unmount()
  })

  it('saves the description written here, a moment after the typing stops', async () => {
    vi.useFakeTimers()
    const store = seed()
    const wrapper = render()
    await wrapper.get('[data-testid="epic-description-mode-write"]').trigger('click')

    wrapper.getComponent({ name: 'AppMarkdownEditor' }).vm.$emit('update:modelValue', 'A sharper concept.')
    await vi.advanceTimersByTimeAsync(800)

    expect(store.saveTaskDescription).toHaveBeenCalledWith(epic, 'A sharper concept.')
    wrapper.unmount()
  })
})
