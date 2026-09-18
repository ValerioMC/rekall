import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NotePlacementsPane from '@/components/console/NotePlacementsPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, RekallDocument, Task, TaskRef } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The column that stands in for the task's while notes are browsed: which tasks the selected
 * note is on, and the opener for the picker that puts it on more. It only decides which task to
 * detach and when the picker shows; the write path is the store's, stubbed here and covered in
 * `console.spec`, and the picker itself is covered in `NoteAssignmentDialog.spec`.
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
  store.deleteNote = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  return mount(NotePlacementsPane, { attachTo: document.body, global: { plugins: [pinia] } })
}

const rowsOf = (wrapper: ReturnType<typeof render>) =>
  wrapper.findAll('[data-testid="note-placement-row"]')

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
    expect(wrapper.find('[data-testid="note-placements-open"]').exists()).toBe(false)
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

  /** The row's body is inert: only its own remove control takes the note anywhere. */
  it('takes the note off a task from the row\'s remove control, never from the row itself', async () => {
    const store = seed(note([builder, wiring]))
    const wrapper = render()

    // Projects sort by title, so Beacon's row comes before Vega's.
    await rowsOf(wrapper)[0]!.trigger('click')
    await flushPromises()
    expect(store.detachNoteFromTask).not.toHaveBeenCalled()

    const remove = wrapper.findAll('[data-testid="note-placement-remove"]')
    expect(remove).toHaveLength(2)
    expect(remove[0]!.attributes('title')).toContain('Take this note off Wiring the adapter')

    await remove[0]!.trigger('click')
    await flushPromises()
    expect(store.detachNoteFromTask).toHaveBeenCalledWith('d1', wiring)
    wrapper.unmount()
  })

  /** Arming the control says on the row what a click will do, before it does it. */
  it('colours the row it is about to empty while the remove control is under the pointer', async () => {
    seed(note([builder, wiring]))
    const wrapper = render()

    const remove = wrapper.find('[data-testid="note-placement-remove"]')
    expect(remove.text()).toContain('Take off')
    await remove.trigger('mouseenter')
    expect(wrapper.find('[data-armed="remove"]').exists()).toBe(true)
    await remove.trigger('mouseleave')
    expect(wrapper.find('[data-armed]').exists()).toBe(false)
    wrapper.unmount()
  })

  /** A note is on at least one task: on the last one the control deletes the note, after asking. */
  it('asks to delete the note instead of taking it off the only task it is on', async () => {
    const store = seed(note([builder]))
    const wrapper = render()

    const only = rowsOf(wrapper)[0]!
    expect(only.text()).toContain('only here')
    expect(wrapper.find('[data-testid="note-placement-remove"]').exists()).toBe(false)
    const del = wrapper.find('[data-testid="note-placement-delete"]')
    expect(del.text()).toContain('Delete note')

    await del.trigger('click')
    await flushPromises()
    const confirm = document.body.querySelector('[role="alertdialog"]')!
    expect(confirm.getAttribute('aria-label')).toBe('Delete cluster.md?')
    expect(confirm.textContent).toContain('Report builder is the only task this note is on')
    expect(store.deleteNote).not.toHaveBeenCalled()

    Array.from(confirm.querySelectorAll('button')).find((b) => b.textContent?.includes('Delete note'))!.click()
    await flushPromises()

    expect(store.deleteNote).toHaveBeenCalledWith('d1')
    expect(store.detachNoteFromTask).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  /** Putting the note somewhere else is the task side's picker, opened here instead of a search. */
  it('opens the same assignment picker the task side uses, and closes it again', async () => {
    seed(note([builder]))
    const wrapper = render()

    expect(wrapper.find('[data-testid="note-assignment"]').exists()).toBe(false)
    const opener = wrapper.find('[data-testid="note-placements-open"]')
    expect(opener.text()).toContain('Put it on a task')

    await opener.trigger('click')
    await flushPromises()

    const picker = wrapper.find('[data-testid="note-assignment"]')
    expect(picker.exists()).toBe(true)
    expect(picker.text()).toContain('Tasks for cluster.md')

    await picker.find('[aria-label="Close"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-testid="note-assignment"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('says on the last task that taking it off deletes the note', () => {
    seed(note([builder]))
    const wrapper = render()

    expect(wrapper.text()).toContain('taking it off deletes the note')
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
