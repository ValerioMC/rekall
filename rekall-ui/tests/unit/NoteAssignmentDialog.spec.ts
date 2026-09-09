import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteAssignmentDialog from '@/components/console/NoteAssignmentDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project, RekallDocument, Task, TaskStatus } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The picker's own decisions: which column it opens on, what each column narrows to, and the
 * one rule it enforces itself rather than leaving to the store, that a note keeps at least one
 * task. The write path is `store.saveNote`, tested in `console.spec`, so it is stubbed here.
 */
const vforge = 'c1' as CompanyId
const northwind = 'c2' as CompanyId

const companies: Company[] = [
  { id: vforge, name: 'vforge', description: null, projectCount: 2, taskCount: 5, updatedAt: '2026-09-01T10:00:00Z' },
  { id: northwind, name: 'northwind', description: null, projectCount: 1, taskCount: 1, updatedAt: '2026-09-01T10:00:00Z' }
]

const rekall = 'p1' as ProjectId
const vega = 'p2' as ProjectId
const beacon = 'p3' as ProjectId

const project = (id: ProjectId, label: string, title: string, companyId: CompanyId): Project => ({
  id,
  label,
  title,
  status: 'ACTIVE',
  description: null,
  blueprintMarkdown: null,
  repoFolder: null,
  companyId,
  companyName: companyId === vforge ? 'vforge' : 'northwind',
  taskCount: 2,
  anchor: `project:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const projects: Project[] = [
  project(rekall, 'rekall', 'Rekall', vforge),
  project(vega, 'vega', 'Vega Platform', vforge),
  project(beacon, 'beacon', 'Beacon', northwind)
]

const noteUx = 't1' as TaskId
const stepConduit = 't2' as TaskId
const reportBuilder = 't3' as TaskId
const wiring = 't4' as TaskId
const oldConsole = 't5' as TaskId

const task = (
  id: TaskId,
  label: string,
  title: string,
  projectId: ProjectId,
  projectLabel: string,
  status: TaskStatus = 'IN_PROGRESS'
): Task => ({
  id,
  label,
  title,
  status,
  description: null,
  projectId,
  projectLabel,
  projectTitle: projectLabel === 'rekall' ? 'Rekall' : projectLabel === 'vega' ? 'Vega Platform' : 'Beacon',
  companyName: projectId === beacon ? 'northwind' : 'vforge',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  anchor: `project:${projectLabel} task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(noteUx, 'note-ux-review', 'Note UX review', rekall, 'rekall'),
  task(stepConduit, 'step-conduit', 'Energy-stream step conduit', rekall, 'rekall'),
  task(reportBuilder, 'report-builder', 'Report builder', vega, 'vega'),
  task(wiring, 'wiring', 'Wiring the adapter', beacon, 'beacon'),
  task(oldConsole, 'old-console', 'Old console shell', rekall, 'rekall', 'DONE')
]

const ref = (t: Task) => ({
  id: t.id,
  label: t.label,
  title: t.title,
  projectLabel: t.projectLabel,
  projectTitle: t.projectTitle,
  companyName: t.companyName,
  anchor: t.anchor
})

function makeDocument(on: Task[]): RekallDocument {
  return {
    id: 'd1' as DocumentId,
    title: 'note-ux-review.md',
    kind: 'notes',
    bodyMarkdown: 'body',
    tasks: on.map(ref),
    updatedAt: '2026-09-05T10:00:00Z'
  }
}

function seed(document: RekallDocument, selectedTaskId: TaskId | null) {
  const store = useConsoleStore()
  store.companies = companies
  store.projects = projects
  store.tasks = tasks
  store.documents = [document]
  store.selectedDocId = document.id
  store.selectedTaskId = selectedTaskId
  store.isLoading = false
  store.saveNote = vi.fn().mockResolvedValue(undefined)
  return store
}

let pinia: Pinia

function render() {
  return mount(NoteAssignmentDialog, {
    attachTo: document.body,
    global: { plugins: [pinia] }
  })
}

describe('NoteAssignmentDialog', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('opens on the project of the task the note was written against', async () => {
    seed(makeDocument([tasks[0]!, tasks[1]!]), stepConduit)
    const wrapper = render()
    await flushPromises()

    // The task column is already the one holding step-conduit, and both Rekall tasks show.
    const taskLabels = wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text())
    expect(taskLabels.join(' ')).toContain('Energy-stream step conduit')
    expect(taskLabels.join(' ')).toContain('Note UX review')
    expect(taskLabels.join(' ')).not.toContain('Report builder')
  })

  it('falls back to the first attached task when none is selected', async () => {
    seed(makeDocument([tasks[2]!]), null)
    const wrapper = render()
    await flushPromises()

    expect(
      wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text()).join(' ')
    ).toContain('Report builder')
  })

  it('narrows projects to the picked company and clears a project from another one', async () => {
    seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    const northwindRow = wrapper
      .findAll('[data-testid="assign-company"]')
      .find((row) => row.text().includes('northwind'))!
    await northwindRow.trigger('click')

    const projectRows = wrapper.findAll('[data-testid="assign-project"]').map((row) => row.text())
    expect(projectRows.join(' ')).toContain('Beacon')
    expect(projectRows.join(' ')).not.toContain('Rekall')
    // The task column empties until a project in the new company is chosen.
    expect(wrapper.findAll('[data-testid="assign-task"]')).toHaveLength(0)
    expect(wrapper.text()).toContain('Pick a project to see its tasks.')
  })

  it('filters the task column by title or label', async () => {
    seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[data-testid="assign-task-filter"]').setValue('conduit')

    const rows = wrapper.findAll('[data-testid="assign-task"]')
    expect(rows).toHaveLength(1)
    expect(rows[0]!.text()).toContain('Energy-stream step conduit')
  })

  it('attaches a task that is not on the note yet', async () => {
    const store = seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    const conduitRow = wrapper
      .findAll('[data-testid="assign-task"]')
      .find((row) => row.text().includes('Energy-stream step conduit'))!
    await conduitRow.trigger('click')

    expect(store.saveNote).toHaveBeenCalledWith('d1', { taskIds: [noteUx, stepConduit] })
  })

  it('refuses to remove the last task the note is on', async () => {
    const store = seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    const onlyRow = wrapper
      .findAll('[data-testid="assign-task"]')
      .find((row) => row.text().includes('Note UX review'))!
    await onlyRow.trigger('click')

    expect(store.saveNote).not.toHaveBeenCalled()
    // And the strip offers no way to drop it either.
    expect(wrapper.text()).toContain('a note needs at least one')
  })

  it('detaches from the strip when the note is on more than one task', async () => {
    const store = seed(makeDocument([tasks[0]!, tasks[1]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    const remove = wrapper.get('[aria-label="Remove this note from Energy-stream step conduit"]')
    await remove.trigger('click')

    expect(store.saveNote).toHaveBeenCalledWith('d1', { taskIds: [noteUx] })
  })

  it('closes on Escape', async () => {
    seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('hides finished tasks in the task column until All is chosen', async () => {
    seed(makeDocument([tasks[0]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    let labels = wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text()).join(' ')
    expect(labels).toContain('Note UX review')
    expect(labels).not.toContain('Old console shell')

    await wrapper.get('[data-testid="assign-scope-all"]').trigger('click')

    labels = wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text()).join(' ')
    expect(labels).toContain('Old console shell')

    await wrapper.get('[data-testid="assign-scope-open"]').trigger('click')
    labels = wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text()).join(' ')
    expect(labels).not.toContain('Old console shell')
  })

  it('offers no scope toggle for a project with nothing finished', async () => {
    seed(makeDocument([tasks[2]!]), reportBuilder)
    const wrapper = render()
    await flushPromises()

    // Vega holds only report-builder, which is in progress.
    expect(wrapper.find('[data-testid="assign-scope-all"]').exists()).toBe(false)
  })

  it('switches to All when the strip reveals a finished task', async () => {
    seed(makeDocument([tasks[0]!, tasks[4]!]), noteUx)
    const wrapper = render()
    await flushPromises()

    const chip = wrapper.get('[title="Show Old console shell in the columns"]')
    await chip.trigger('click')

    const labels = wrapper.findAll('[data-testid="assign-task"]').map((row) => row.text()).join(' ')
    expect(labels).toContain('Old console shell')
    expect(wrapper.get('[data-testid="assign-scope-all"]').attributes('aria-pressed')).toBe('true')
  })
})
