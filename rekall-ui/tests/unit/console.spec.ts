import { beforeEach, describe, expect, it, vi } from 'vitest'
import { updateProject } from '@/api/catalog.api'
import { setActivePinia, createPinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskInput } from '@/api/catalog.api'
import type { Company, Project, RekallDocument, Task, Wrapup } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId, WrapupId } from '@/model/branded'

/**
 * The console's reading rules, which is where the screen's behaviour actually lives: what the
 * project scope hides, what the search finds, and what a note being on several tasks means for
 * both. Mounting the panes would test Vue; this tests the decisions.
 */
const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 1, taskCount: 2, updatedAt: '2026-08-12T10:00:00Z' },
  { id: globex, name: 'globex', description: null, projectCount: 1, taskCount: 1, updatedAt: '2026-08-12T10:00:00Z' }
]

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId

const projects: Project[] = [
  { id: vega, label: 'vega', title: 'Vega Platform', status: 'ACTIVE', description: null, blueprintMarkdown: null,
    repoFolder: null, companyId: acme, companyName: 'acme', taskCount: 2, anchor: 'project:vega', updatedAt: '2026-08-12T10:00:00Z' },
  { id: beacon, label: 'beacon', title: 'Beacon', status: 'ACTIVE', description: null, blueprintMarkdown: null,
    repoFolder: null, companyId: globex, companyName: 'globex', taskCount: 1, anchor: 'project:beacon', updatedAt: '2026-08-12T10:00:00Z' }
]

const validator = 't1' as TaskId
const retry = 't2' as TaskId
const wiring = 't3' as TaskId

const task = (
  id: TaskId,
  label: string,
  title: string,
  status: Task['status'],
  projectId: ProjectId,
  projectLabel: string,
  projectTitle: string,
  companyName: string
): Task => ({
  id,
  label,
  title,
  status,
  description: null,
  projectId,
  projectLabel,
  projectTitle,
  companyName,
  projectRepoFolder: null,
  documentCount: 1,
  hasWrapup: id === validator,
  anchor: `project:${projectLabel} task:${label}`,
  updatedAt: '2026-08-12T10:00:00Z'
})

const tasks: Task[] = [
  task(validator, 'report-builder', 'Report builder', 'IN_PROGRESS', vega, 'vega', 'Vega Platform', 'acme'),
  task(retry, 'retry-policy', 'Retry policy', 'TODO', vega, 'vega', 'Vega Platform', 'acme'),
  task(wiring, 'wiring', 'Wiring the adapter', 'DONE', beacon, 'beacon', 'Beacon', 'globex')
]

const ref = (id: TaskId, label: string, title: string, projectLabel: string, companyName = 'acme') => ({
  id,
  label,
  title,
  projectLabel,
  projectTitle: projectLabel === 'vega' ? 'Vega Platform' : 'Beacon',
  companyName,
  anchor: `project:${projectLabel} task:${label}`
})

const documents: RekallDocument[] = [
  {
    id: 'd1' as DocumentId,
    title: 'CONTEXT.md',
    kind: 'context',
    bodyMarkdown: 'Il workflow parte da POST /api/v1/pipelines',
    tasks: [ref(validator, 'report-builder', 'Report builder', 'vega')],
    updatedAt: '2026-08-12T12:00:00Z'
  },
  {
    // The shape the whole model change exists for: one note, three tasks.
    id: 'd2' as DocumentId,
    title: 'kmaster14.md',
    kind: 'notes',
    bodyMarkdown: 'Accesso via bastion',
    tasks: [
      ref(validator, 'report-builder', 'Report builder', 'vega'),
      ref(retry, 'retry-policy', 'Retry policy', 'vega')
    ],
    updatedAt: '2026-08-12T13:00:00Z'
  },
  {
    id: 'd3' as DocumentId,
    title: 'onboarding.md',
    kind: 'notes',
    bodyMarkdown: 'brew install openjdk',
    tasks: [ref(wiring, 'wiring', 'Wiring the adapter', 'beacon', 'globex')],
    updatedAt: '2026-08-12T09:00:00Z'
  }
]

/**
 * One task has a wrapup and it is deliberately older than one of that task's notes, because
 * "written before the notes it summarises" is the state the console has to be able to report.
 */
const wrapups: Wrapup[] = [
  {
    id: 'w1' as WrapupId,
    taskId: validator,
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: 'project:vega task:report-builder',
    bodyMarkdown: '## Stato\n\nIl builder gira su POST /api/v1/pipelines.',
    writtenBy: 'CLAUDE',
    createdAt: '2026-08-12T11:00:00Z',
    updatedAt: '2026-08-12T12:30:00Z'
  }
]

const saveWrapup = vi.fn(async (_taskId: TaskId, bodyMarkdown: string) => ({
  ...wrapups[0]!,
  bodyMarkdown,
  writtenBy: 'HAND' as const,
  updatedAt: '2026-08-12T14:00:00Z'
}))
const deleteWrapup = vi.fn(async () => undefined)

const updateTask = vi.fn(async (id: TaskId, input: TaskInput) => ({
  ...(tasks.find((candidate) => candidate.id === id) ?? tasks[0]!),
  ...input
}))

vi.mock('@/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => companies),
  fetchProjects: vi.fn(async () => projects),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  fetchTasks: vi.fn(async () => tasks),
  createProject: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  createTask: vi.fn(),
  updateTask: (...args: Parameters<typeof updateTask>) => updateTask(...args),
  deleteTask: vi.fn()
}))

vi.mock('@/api/documents.api', () => ({
  fetchAllDocuments: vi.fn(async () => documents),
  createDocument: vi.fn(),
  updateDocument: vi.fn(),
  deleteDocument: vi.fn()
}))

vi.mock('@/api/wrapups.api', () => ({
  fetchWrapups: vi.fn(async () => wrapups),
  saveWrapup: (...args: unknown[]) => saveWrapup(...(args as [TaskId, string])),
  deleteWrapup: (...args: unknown[]) => deleteWrapup(...(args as []))
}))

vi.mock('@/api/time-entries.api', () => ({
  fetchTimeEntries: vi.fn(async () => []),
  startTimeEntry: vi.fn(),
  stopTimeEntry: vi.fn(),
  editTimeEntry: vi.fn(),
  deleteTimeEntry: vi.fn()
}))

describe('console store', () => {
  let store: ReturnType<typeof useConsoleStore>

  beforeEach(async () => {
    updateTask.mockClear()
    saveWrapup.mockClear()
    deleteWrapup.mockClear()
    setActivePinia(createPinia())
    store = useConsoleStore()
    await store.load()
  })

  it('narrows from everything, to a company, to one of its projects', () => {
    expect(store.visibleTasks).toHaveLength(3)

    store.setScope(acme)
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['report-builder', 'retry-policy'])

    store.setScope(acme, vega)
    expect(store.visibleTasks).toHaveLength(2)
    expect(store.scopeName).toBe('acme / Vega Platform')
    expect(store.scopeAnchor).toBe('project:vega')

    store.setScope(null)
    expect(store.visibleTasks).toHaveLength(3)
    expect(store.scopeName).toBe('All work')
  })

  it('finds a task by either half of its anchor', () => {
    store.filter = 'vega/retry'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['retry-policy'])

    store.filter = 'task:report-builder'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['report-builder'])
  })

  /**
   * The label is what you type after `/rk` and the title is what you called it out loud. Both
   * have to find the row, or one of them is a name you cannot search by.
   */
  it('finds a task by its title as well as by its label', () => {
    store.filter = 'Wiring the adapter'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['wiring'])

    store.filter = 'wiring'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['wiring'])
  })

  it('searches note bodies, not only their titles', () => {
    store.navMode = 'notes'
    store.filter = 'bastion'
    expect(store.visibleDocuments.map((document) => document.title)).toEqual(['kmaster14.md'])
  })

  /**
   * The case that made this necessary: scoped to one project, searching for something that
   * lives in another. Without the notice the answer looks like "you never wrote it".
   */
  it('reports the matches the project scope is hiding', () => {
    store.setScope(acme)
    store.filter = 'wiring'

    expect(store.visibleTasks).toHaveLength(0)
    expect(store.elsewhere).toEqual({ count: 1, names: ['globex'] })

    store.setScope(null)
    expect(store.visibleTasks).toHaveLength(1)
    expect(store.elsewhere).toBeNull()
  })

  it('says nothing about elsewhere when the search is already global', () => {
    store.filter = 'wiring'
    expect(store.elsewhere).toBeNull()
  })

  it('lists a shared note under every task it is attached to', () => {
    store.selectTask(validator)
    expect(store.taskDocuments.map((document) => document.title)).toEqual([
      'CONTEXT.md',
      'kmaster14.md'
    ])

    store.selectTask(retry)
    expect(store.taskDocuments.map((document) => document.title)).toEqual(['kmaster14.md'])
  })

  it('opens the first note of a task when the task is picked', () => {
    store.selectTask(retry)
    expect(store.selectedDocId).toBe('d2')
  })

  /** Picking a note from the Notes list has to bring its task along, or the panes disagree. */
  it('follows a note back to a task it belongs to', () => {
    store.selectTask(validator)
    store.selectDocument('d3' as DocumentId)

    expect(store.selectedTaskId).toBe(wiring)
  })

  it('offers the most recently written notes first', () => {
    expect(store.recentDocuments.map((document) => document.title)).toEqual([
      'kmaster14.md',
      'CONTEXT.md',
      'onboarding.md'
    ])
  })

  it('keeps the selection when the new scope still contains it', () => {
    store.selectTask(validator)
    store.setScope(acme)
    expect(store.selectedTaskId).toBe(validator)
  })

  it('moves the selection into the new scope when the old one falls outside it', () => {
    store.selectTask(wiring)
    store.setScope(acme)
    expect(store.selectedTaskId).toBe(validator)
  })

  describe('the wrapup', () => {
    /** One per task, and the task in view is what decides which one is on screen. */
    it('shows the wrapup of the selected task, and nothing for a task without one', () => {
      store.selectTask(validator)
      expect(store.selectedWrapup?.anchor).toBe('project:vega task:report-builder')

      store.selectTask(retry)
      expect(store.selectedWrapup).toBeNull()
    })

    /**
     * A wrapup goes stale silently, which is the one way it can start lying. Notes written
     * after it are the cheap half of that, and the console counts them rather than judging.
     */
    it('counts the notes written since the wrapup was', () => {
      store.selectTask(validator)
      // kmaster14.md is 13:00, the wrapup is 12:30; CONTEXT.md at 12:00 is not.
      expect(store.wrapupIsBehind).toBe(1)

      store.selectTask(retry)
      expect(store.wrapupIsBehind).toBe(0)
    })

    it('opens the wrapup pane on the task in view, and leaves it when another is picked', () => {
      store.selectTask(validator)
      store.openWrapup()
      expect(store.paneFocus).toBe('wrapup')

      store.selectTask(retry)
      expect(store.paneFocus).toBe('note')
    })

    /** What the keyboard does. A one-way door would need a second key to undo it. */
    it('toggles the pane, and does nothing at all with no task in view', () => {
      store.toggleWrapup()
      expect(store.paneFocus).toBe('note')

      store.selectTask(validator)
      store.toggleWrapup()
      expect(store.paneFocus).toBe('wrapup')
      store.toggleWrapup()
      expect(store.paneFocus).toBe('note')
    })

    /** Opening a note is how you leave the wrapup, so the two panes never both claim to be on. */
    it('returns to the note pane when a note is selected', () => {
      store.selectTask(validator)
      store.openWrapup()
      store.selectDocument('d1' as DocumentId)

      expect(store.paneFocus).toBe('note')
    })

    it('sends the whole body and keeps what came back', async () => {
      store.selectTask(validator)
      await store.saveWrapupBody(validator, '## Stato\n\nRiscritto a mano.')

      expect(saveWrapup).toHaveBeenCalledWith(validator, '## Stato\n\nRiscritto a mano.')
      expect(store.selectedWrapup?.bodyMarkdown).toBe('## Stato\n\nRiscritto a mano.')
      // The console shows whose words are on screen, and they are now yours.
      expect(store.selectedWrapup?.writtenBy).toBe('HAND')
      expect(store.saveState).toBe('saved')
    })

    it('drops the wrapup and returns to the notes when it is deleted', async () => {
      store.selectTask(validator)
      store.openWrapup()
      await store.removeWrapup(validator)

      expect(deleteWrapup).toHaveBeenCalledWith(validator)
      expect(store.selectedWrapup).toBeNull()
      expect(store.paneFocus).toBe('note')
    })
  })

  /**
   * A task row carries a copy of its project's folder, because the button that opens a session
   * lives on the task. Saving the folder has to reach the rows already loaded, or that button
   * goes on saying there is nowhere to open until the window is reloaded.
   */
  it('carries a saved project folder onto the tasks already in view', async () => {
    vi.mocked(updateProject).mockResolvedValue({
      ...projects[0]!,
      repoFolder: '/Users/someone/Projects/vega'
    })

    await store.saveProjectRepoFolder(vega, '  /Users/someone/Projects/vega  ')

    expect(vi.mocked(updateProject).mock.calls[0]?.[1]).toMatchObject({
      repoFolder: '/Users/someone/Projects/vega'
    })
    expect(
      store.tasks.filter((task) => task.projectId === vega).map((task) => task.projectRepoFolder)
    ).toEqual(['/Users/someone/Projects/vega', '/Users/someone/Projects/vega'])
    expect(
      store.tasks.filter((task) => task.projectId !== vega).map((task) => task.projectRepoFolder)
    ).toEqual([null])
  })

  /**
   * A status change sends the record back whole. Dropping the label out of that payload would
   * blank the column the anchor resolves on, which the endpoint would then reject or, worse,
   * accept.
   */
  it('keeps the label and the title when only the status changes', async () => {
    await store.setTaskStatus(validator, 'DONE')

    expect(updateTask).toHaveBeenCalledWith(
      validator,
      expect.objectContaining({
        label: 'report-builder',
        title: 'Report builder',
        status: 'DONE',
        projectId: vega
      })
    )
  })

  /**
   * The description is edited where it is read, on the pane, and carries the same obligation as
   * a status change: everything else about the record goes back untouched.
   */
  describe('the description', () => {
    it('saves it without moving the label, the title or the status', async () => {
      await store.saveTaskDescription(validator, 'Builds the weekly report from the pipeline runs.')

      expect(updateTask).toHaveBeenCalledWith(validator, {
        label: 'report-builder',
        title: 'Report builder',
        status: 'IN_PROGRESS',
        description: 'Builds the weekly report from the pipeline runs.',
        projectId: vega
      })
      expect(store.tasks.find((task) => task.id === validator)?.description).toBe(
        'Builds the weekly report from the pipeline runs.'
      )
    })

    /** Emptied means there is none, not that there is one made of spaces. */
    it('stores a blank one as no description at all', async () => {
      await store.saveTaskDescription(validator, 'Something to erase.')
      await store.saveTaskDescription(validator, '   ')

      expect(updateTask).toHaveBeenLastCalledWith(
        validator,
        expect.objectContaining({ description: null })
      )
      expect(store.tasks.find((task) => task.id === validator)?.description).toBeNull()
    })

    /** The pane it is written in is reached and left on one key, like the wrapup's. */
    it('opens and closes its pane without touching the note in view', async () => {
      store.selectTask(validator)
      expect(store.paneFocus).toBe('note')

      store.toggleDescription()
      expect(store.paneFocus).toBe('description')

      store.toggleDescription()
      expect(store.paneFocus).toBe('note')
      expect(store.selectedDocId).toBe('d1')
    })

    /** Autosave fires on a pause, not on a change, so it lands on text that is already saved. */
    it('sends nothing when the text is what is already stored', async () => {
      await store.saveTaskDescription(validator, '')

      expect(updateTask).not.toHaveBeenCalled()
      expect(store.saveState).toBe('saved')
    })
  })
})
