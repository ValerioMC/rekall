import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteComposerPane from '@/components/console/NoteComposerPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project, Task } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/**
 * The column that starts a note from the Notes side: a name, where it lives, and the tasks of
 * that scope it goes on. It only decides what to send; `store.createNote` is stubbed here and
 * covered in `console.spec`.
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

const acme = 'c1' as CompanyId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 2, taskCount: 4, updatedAt: '2026-09-01T10:00:00Z' }
]

const project = (id: ProjectId, label: string, title: string): Project => ({
  id,
  label,
  title,
  status: 'ACTIVE',
  icon: 'folder',
  description: null,
  blueprintMarkdown: null,
  repoFolder: null,
  autoCommit: false,
  companyId: acme,
  companyName: 'acme',
  taskCount: 2,
  anchor: `project:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const projects: Project[] = [project(vega, 'vega', 'Vega Platform'), project(beacon, 'beacon', 'Beacon')]

let pinia: Pinia

function seed(taskInView: TaskId | null) {
  const store = useConsoleStore()
  store.companies = companies
  store.projects = projects
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

  it('starts in the project of the task in view, offering that project\'s live tasks', () => {
    seed(builder)
    const wrapper = render()

    const rows = rowsOf(wrapper)
    expect(rows.map((row) => row.text())).toEqual([
      expect.stringContaining('Report builder'),
      expect.stringContaining('Retry policy')
    ])
    expect(rows.map((row) => row.attributes('data-attached'))).toEqual(['true', 'false'])
    expect(wrapper.get('[data-testid="note-composer-scope-project"]').attributes('aria-checked')).toBe('true')
    expect(wrapper.get('[data-testid="note-composer-count"]').text()).toContain('In Vega Platform · on 1 task')
    expect(document.activeElement).toBe(wrapper.find('[data-testid="note-composer-title"]').element)
    wrapper.unmount()
  })

  it('creates a note on no task, in the scope it starts in', async () => {
    const store = seed(null)
    const wrapper = render()

    await wrapper.find('[data-testid="note-composer-title"]').setValue('conventions.md')
    await wrapper.find('[data-testid="note-composer-title"]').trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith({
      scope: { kind: 'COMPANY', id: acme },
      taskIds: [],
      title: 'conventions.md'
    })
    wrapper.unmount()
  })

  it('creates the note with its name and scope on every ticked task', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.find('[data-testid="note-composer-title"]').setValue('cluster.md')
    await rowTitled(wrapper, 'Retry policy').trigger('click')
    await wrapper.find('[data-testid="note-composer-create"]').trigger('click')
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith({
      scope: { kind: 'PROJECT', id: vega },
      taskIds: [builder, retry],
      title: 'cluster.md'
    })
    wrapper.unmount()
  })

  it('widens to the company and finds its other projects\' tasks by typing', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.get('[data-testid="note-composer-scope-company"]').trigger('click')
    await type(wrapper, 'wiring')
    const filter = wrapper.find('[data-testid="note-composer-filter"]')
    await filter.trigger('keydown', { key: 'Enter' })
    await filter.trigger('keydown', { key: 'Enter', metaKey: true })
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith({
      scope: { kind: 'COMPANY', id: acme },
      taskIds: [builder, wiring],
      title: ''
    })
    wrapper.unmount()
  })

  it('lets go of a tick the narrower scope cannot hold', async () => {
    const store = seed(builder)
    const wrapper = render()

    await wrapper.get('[data-testid="note-composer-scope-company"]').trigger('click')
    await type(wrapper, 'wiring')
    await rowsOf(wrapper)[0]!.trigger('click')
    await wrapper.get('[data-testid="note-composer-scope-project"]').trigger('click')
    await flushPromises()
    await wrapper.find('[data-testid="note-composer-create"]').trigger('click')
    await flushPromises()

    expect(store.createNote).toHaveBeenCalledWith({
      scope: { kind: 'PROJECT', id: vega },
      taskIds: [builder],
      title: ''
    })
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
