import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteComposerPane from '@/components/console/NoteComposerPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Task } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/**
 * The column that starts a note from the Notes side: a name and the tasks it goes on, in one
 * place. It only decides what to send; `store.createNote` is stubbed here and covered in
 * `console.spec`.
 */
const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId

const builder = 't1' as TaskId
const retry = 't2' as TaskId
const wiring = 't3' as TaskId
const archive = 't4' as TaskId

const task = (
  id: TaskId,
  label: string,
  title: string,
  status: Task['status'],
  projectId: ProjectId
): Task => ({
  id,
  label,
  title,
  status,
  description: null,
  projectId,
  projectLabel: projectId === vega ? 'vega' : 'beacon',
  projectTitle: projectId === vega ? 'Vega Platform' : 'Beacon',
  companyName: 'acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: false,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  anchor: `project:${projectId === vega ? 'vega' : 'beacon'} task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(builder, 'report-builder', 'Report builder', 'IN_PROGRESS', vega),
  task(retry, 'retry-policy', 'Retry policy', 'TODO', vega),
  task(wiring, 'wiring', 'Wiring the adapter', 'TODO', beacon),
  task(archive, 'archive', 'Archive the old rows', 'DONE', beacon)
]

const companies: Company[] = [
  { id: 'c1' as CompanyId, name: 'acme', description: null, projectCount: 2, taskCount: 4, updatedAt: '2026-09-01T10:00:00Z' }
]

let pinia: Pinia

function seed(taskInView: TaskId | null) {
  const store = useConsoleStore()
  store.companies = companies
  store.tasks = tasks
  store.documents = []
  store.selectedTaskId = taskInView
  store.navMode = 'notes'
  store.noteComposerOpen = true
  store.isLoading = false
  store.createNote = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  return mount(NoteComposerPane, { attachTo: document.body, global: { plugins: [pinia] } })
}

const rowsOf = (wrapper: ReturnType<typeof render>) =>
  wrapper.findAll('[data-testid="note-placement-row"]')

const rowTitled = (wrapper: ReturnType<typeof render>, title: string) =>
  rowsOf(wrapper).find((row) => row.text().includes(title))!

async function type(wrapper: ReturnType<typeof render>, needle: string): Promise<void> {
  await wrapper.find('[data-testid="note-composer-filter"]').setValue(needle)
  await flushPromises()
}

describe('NoteComposerPane', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('starts on the task in view, with the live tasks in scope offered underneath', () => {
    seed(builder)
    const wrapper = render()

    const rows = rowsOf(wrapper)
    expect(rows.map((row) => row.attributes('data-attached'))).toEqual(['false', 'true', 'false'])
    expect(rows.map((row) => row.text())).toEqual([
      expect.stringContaining('Wiring the adapter'),
      expect.stringContaining('Report builder'),
      expect.stringContaining('Retry policy')
    ])
    expect(wrapper.text()).not.toContain('Archive the old rows')
    expect(wrapper.text()).toContain('On 1 task')
    expect(document.activeElement).toBe(wrapper.find('[data-testid="note-composer-title"]').element)
    wrapper.unmount()
  })

  it('will not create a note that is on no task', async () => {
    const store = seed(null)
    const wrapper = render()

    const create = wrapper.find('[data-testid="note-composer-create"]')
    expect(create.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Tick at least one task')

    await wrapper.find('[data-testid="note-composer-title"]').trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(store.createNote).not.toHaveBeenCalled()
    expect(document.activeElement).toBe(wrapper.find('[data-testid="note-composer-filter"]').element)
    wrapper.unmount()
  })

  it('creates the note with its name on every ticked task', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.find('[data-testid="note-composer-title"]').setValue('cluster.md')
    await rowTitled(wrapper, 'Wiring the adapter').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('On 2 tasks')

    await wrapper.find('[data-testid="note-composer-create"]').trigger('click')
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith([builder, wiring], 'cluster.md')
    wrapper.unmount()
  })

  it('creates on Enter from the name once a task is ticked', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.find('[data-testid="note-composer-title"]').setValue('cluster.md')
    await wrapper.find('[data-testid="note-composer-title"]').trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith([builder], 'cluster.md')
    wrapper.unmount()
  })

  it('finds any task by typing and ticks it with the arrows and Enter', async () => {
    const store = seed(null)
    const wrapper = render()

    await type(wrapper, 'archive')
    expect(rowsOf(wrapper)).toHaveLength(1)

    const filter = wrapper.find('[data-testid="note-composer-filter"]')
    await filter.trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(rowsOf(wrapper)[0]!.attributes('data-attached')).toBe('true')

    await filter.trigger('keydown', { key: 'Enter', metaKey: true })
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith([archive], '')
    wrapper.unmount()
  })

  it('keeps ticked tasks in the list when the search is cleared', async () => {
    seed(null)
    const wrapper = render()

    await type(wrapper, 'archive')
    await rowsOf(wrapper)[0]!.trigger('click')
    await type(wrapper, '')

    const rows = rowsOf(wrapper)
    expect(rows[0]!.text()).toContain('Archive the old rows')
    expect(rows[0]!.attributes('data-attached')).toBe('true')
    wrapper.unmount()
  })

  it('closes on Escape without creating anything', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.find('[data-testid="note-composer-title"]').trigger('keydown', { key: 'Escape' })
    await flushPromises()

    expect(store.noteComposerOpen).toBe(false)
    expect(store.createNote).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
