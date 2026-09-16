import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NotePlacementsPane from '@/components/console/NotePlacementsPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, RekallDocument, Task, TaskRef } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The column that stands in for the task's while notes are browsed: which tasks the selected
 * note is on, and the search that widens it to the ones it could be put on. It only decides
 * which task to attach or detach; the write path is the store's, stubbed here and covered in
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
  anchor: `project:${projectId === vega ? 'vega' : 'beacon'} task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(builder, 'report-builder', 'Report builder', 'IN_PROGRESS', vega),
  task(retry, 'retry-policy', 'Retry policy', 'TODO', vega),
  task(wiring, 'wiring', 'Wiring the adapter', 'TODO', beacon),
  task(archive, 'archive', 'Archive the old rows', 'DONE', beacon)
]

const refOf = (id: TaskId): TaskRef => {
  const found = tasks.find((candidate) => candidate.id === id)!
  return {
    id,
    label: found.label,
    title: found.title,
    projectLabel: found.projectLabel,
    projectTitle: found.projectTitle,
    companyName: found.companyName,
    anchor: found.anchor
  }
}

const note = (on: TaskId[]): RekallDocument => ({
  id: 'd1' as DocumentId,
  title: 'cluster.md',
  kind: 'notes',
  bodyMarkdown: 'Accesso via bastion',
  tasks: on.map(refOf),
  updatedAt: '2026-09-01T10:00:00Z'
})

const companies: Company[] = [
  { id: 'c1' as CompanyId, name: 'acme', description: null, projectCount: 2, taskCount: 4, updatedAt: '2026-09-01T10:00:00Z' }
]

let pinia: Pinia

function seed(document: RekallDocument | null) {
  const store = useConsoleStore()
  store.companies = companies
  store.tasks = tasks
  store.documents = document ? [document] : []
  store.selectedDocId = document?.id ?? null
  store.navMode = 'notes'
  store.isLoading = false
  store.attachNoteToTask = vi.fn().mockResolvedValue(undefined)
  store.detachNoteFromTask = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  return mount(NotePlacementsPane, { attachTo: document.body, global: { plugins: [pinia] } })
}

const rowsOf = (wrapper: ReturnType<typeof render>) =>
  wrapper.findAll('[data-testid="note-placement-row"]')

async function type(wrapper: ReturnType<typeof render>, needle: string): Promise<void> {
  await wrapper.find('[data-testid="note-placements-filter"]').setValue(needle)
  await flushPromises()
}

describe('NotePlacementsPane', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('asks for a note when none is picked', () => {
    seed(null)
    const wrapper = render()

    expect(wrapper.text()).toContain('Pick a note to see which tasks it is on')
    expect(wrapper.find('[data-testid="note-placements-filter"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('lists only the tasks the note is on, grouped by project, and says how many', () => {
    seed(note([builder, wiring]))
    const wrapper = render()

    const rows = rowsOf(wrapper)
    expect(rows).toHaveLength(2)
    expect(rows.map((row) => row.attributes('data-attached'))).toEqual(['true', 'true'])
    expect(wrapper.text()).toContain('On 2 tasks')
    expect(wrapper.text()).toContain('Vega Platform')
    expect(wrapper.text()).toContain('Beacon')
    expect(wrapper.text()).not.toContain('Retry policy')
    wrapper.unmount()
  })

  it('takes the note off a task it is on with one click', async () => {
    const store = seed(note([builder, wiring]))
    const wrapper = render()

    // Projects sort by title, so Beacon's row comes before Vega's.
    await rowsOf(wrapper)[0]!.trigger('click')
    await flushPromises()

    expect(store.detachNoteFromTask).toHaveBeenCalledWith('d1', wiring)
    wrapper.unmount()
  })

  /** A note is on at least one task: the last row is inert and says why. */
  it('will not take the note off the only task it is on', async () => {
    const store = seed(note([builder]))
    const wrapper = render()

    const only = rowsOf(wrapper)[0]!
    expect(only.attributes('aria-disabled')).toBe('true')
    expect(only.text()).toContain('only here')

    await only.trigger('click')
    await flushPromises()

    expect(store.detachNoteFromTask).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('widens to every matching task when typing, the ones it is on first in their project', async () => {
    seed(note([wiring]))
    const wrapper = render()

    await type(wrapper, 'r')

    const rows = rowsOf(wrapper)
    expect(rows.map((row) => row.attributes('data-attached'))).toEqual(['true', 'false', 'false', 'false'])
    expect(rows[0]!.text()).toContain('Wiring the adapter')
    wrapper.unmount()
  })

  it('puts the note on a task found by search with one click', async () => {
    const store = seed(note([builder]))
    const wrapper = render()

    await type(wrapper, 'retry')

    const rows = rowsOf(wrapper)
    expect(rows).toHaveLength(1)
    await rows[0]!.trigger('click')
    await flushPromises()

    expect(store.attachNoteToTask).toHaveBeenCalledWith('d1', retry)
    wrapper.unmount()
  })

  it('walks the matches with the arrows and flips the highlighted one on Enter', async () => {
    const store = seed(note([builder]))
    const wrapper = render()

    await type(wrapper, 'beacon')
    const field = wrapper.find('[data-testid="note-placements-filter"]')
    await field.trigger('keydown', { key: 'ArrowDown' })
    await field.trigger('keydown', { key: 'Enter' })
    await flushPromises()

    // "beacon" matches its two tasks, wiring then archive; one step down lands on archive.
    expect(store.attachNoteToTask).toHaveBeenCalledWith('d1', archive)
    wrapper.unmount()
  })

  it('says when nothing matches', async () => {
    seed(note([builder]))
    const wrapper = render()

    await type(wrapper, 'zzz')

    expect(wrapper.find('[data-testid="note-placements-empty"]').text()).toContain('No task matches')
    wrapper.unmount()
  })

  it('folds finished tasks behind a count until asked', async () => {
    seed(note([builder, archive]))
    const wrapper = render()

    expect(rowsOf(wrapper)).toHaveLength(1)
    const toggle = wrapper.find('[data-testid="note-placements-done-toggle"]')
    expect(toggle.text()).toContain('1 done')

    await toggle.trigger('click')
    expect(rowsOf(wrapper)).toHaveLength(2)
    wrapper.unmount()
  })

  /** The one way out is explicit: the arrow opens the task on the Tasks side, note in tow. */
  it('opens a task on the Tasks side only from its own arrow', async () => {
    const store = seed(note([builder, wiring]))
    store.selectedTaskId = retry
    const wrapper = render()

    await wrapper.findAll('[data-testid="note-placement-open"]')[0]!.trigger('click')
    await flushPromises()

    expect(store.navMode).toBe('tasks')
    expect(store.selectedTaskId).toBe(wiring)
    expect(store.selectedDocId).toBe('d1')
    expect(store.detachNoteFromTask).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
