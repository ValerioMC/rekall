import { beforeEach, describe, expect, it, vi } from 'vitest'
import { updateProject } from '@/api/catalog.api'
import { fetchTimeEntries } from '@/api/time-entries.api'
import { setActivePinia, createPinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskInput } from '@/api/catalog.api'
import type { Company, Project, RekallDocument, Task, TaskStep, Wrapup } from '@/model/catalog'
import type {
  CompanyId,
  DocumentId,
  ProjectId,
  TaskId,
  TaskStepId,
  WrapupId
} from '@/model/branded'

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
  autoWrapup: false,
  wrapupDirective: null,
  projectId,
  projectLabel,
  projectTitle,
  companyName,
  projectRepoFolder: null,
  documentCount: 1,
  stepCount: 0,
  stepsDone: 0,
  hasWrapup: id === validator,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
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

/**
 * A checklist on the task that also has a wrapup, half of it done. Both surfaces on one task is
 * the case that matters: they answer different questions and neither can be read off the other.
 */
const steps: TaskStep[] = [
  {
    id: 's1' as TaskStepId,
    taskId: validator,
    title: 'Aggregate the rows',
    bodyMarkdown: null,
    state: 'DONE',
    done: true,
    runningAt: null,
    claimedAt: null,
    // Deliberately after the wrapup's own timestamp: "this step finished and the wrapup has
    // not been rewritten since" is the state the console has to be able to report.
    doneAt: '2026-08-12T13:30:00Z',
    position: 0,
    createdAt: '2026-08-12T10:00:00Z',
    updatedAt: '2026-08-12T11:00:00Z'
  },
  {
    id: 's2' as TaskStepId,
    taskId: validator,
    title: 'Write the tests',
    bodyMarkdown: 'Un caso per settimana vuota.',
    state: 'OPEN',
    done: false,
    runningAt: null,
    claimedAt: null,
    doneAt: null,
    position: 1,
    createdAt: '2026-08-12T10:00:00Z',
    updatedAt: '2026-08-12T10:00:00Z'
  }
]

const createStep = vi.fn(async (taskId: TaskId, title: string) => ({
  id: 's3' as TaskStepId,
  taskId,
  title,
  bodyMarkdown: null,
  state: 'OPEN' as const,
  done: false,
  runningAt: null,
  claimedAt: null,
  doneAt: null,
  position: 2,
  createdAt: '2026-08-12T16:00:00Z',
  updatedAt: '2026-08-12T16:00:00Z'
}))

/** Mirrors the server: a `done` in the patch also settles the step's state. */
const patchStep = vi.fn(async (id: TaskStepId, patch: Partial<TaskStep>) => {
  const merged = { ...steps.find((step) => step.id === id)!, ...patch }
  if ('done' in patch) merged.state = patch.done ? 'DONE' : 'OPEN'
  return merged
})

const moveStep = vi.fn(async () => [
  { ...steps[1]!, position: 0 },
  { ...steps[0]!, position: 1 }
])

const deleteStep = vi.fn(async () => undefined)

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

vi.mock('@/api/steps.api', () => ({
  fetchSteps: vi.fn(async () => steps),
  createStep: (...args: unknown[]) => createStep(...(args as [TaskId, string])),
  patchStep: (...args: unknown[]) => patchStep(...(args as [TaskStepId, Partial<TaskStep>])),
  moveStep: (...args: unknown[]) => moveStep(...(args as [])),
  deleteStep: (...args: unknown[]) => deleteStep(...(args as []))
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
    createStep.mockClear()
    patchStep.mockClear()
    moveStep.mockClear()
    deleteStep.mockClear()
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

    /**
     * A wrapup written elsewhere (a hosted session, an MCP call) arrives on the step feed. The
     * store adopts it in place so the pane reflects it without a reload, and never moves the
     * pane the reader is on.
     */
    describe('arriving over the feed', () => {
      const feedWrapup = (taskId: TaskId, over: Partial<Wrapup> = {}): Wrapup => ({
        id: 'w9' as WrapupId,
        taskId,
        taskLabel: 'retry-policy',
        taskTitle: 'Retry policy',
        projectLabel: 'vega',
        anchor: 'project:vega task:retry-policy',
        bodyMarkdown: '## Stato\n\nScritto da una sessione.',
        writtenBy: 'CLAUDE',
        createdAt: '2026-08-12T15:00:00Z',
        updatedAt: '2026-08-12T15:00:00Z',
        ...over
      })

      it('adds a wrapup for a task that had none and flips its hasWrapup', () => {
        store.applyWrapupEvent({ taskId: retry, wrapup: feedWrapup(retry), deleted: false })

        store.selectTask(retry)
        expect(store.selectedWrapup?.bodyMarkdown).toBe('## Stato\n\nScritto da una sessione.')
        expect(store.tasks.find((task) => task.id === retry)?.hasWrapup).toBe(true)
      })

      it('replaces the body and author of a wrapup already on screen', () => {
        store.selectTask(validator)
        store.applyWrapupEvent({
          taskId: validator,
          wrapup: feedWrapup(validator, {
            id: 'w1' as WrapupId,
            bodyMarkdown: '## Stato\n\nRiscritto da Claude.'
          }),
          deleted: false
        })

        expect(store.selectedWrapup?.bodyMarkdown).toBe('## Stato\n\nRiscritto da Claude.')
        expect(store.selectedWrapup?.writtenBy).toBe('CLAUDE')
      })

      it('drops a deleted wrapup without moving the pane', () => {
        store.selectTask(validator)
        store.openWrapup()

        store.applyWrapupEvent({ taskId: validator, wrapup: null, deleted: true })

        expect(store.selectedWrapup).toBeNull()
        expect(store.tasks.find((task) => task.id === validator)?.hasWrapup).toBe(false)
        expect(store.paneFocus).toBe('wrapup')
      })
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
   * The backend closes any open session when a task is marked done, so the running dock has to
   * reread the sessions or it would keep showing a timer that has already stopped.
   */
  it('rereads the sessions when a task is marked done', async () => {
    vi.mocked(fetchTimeEntries).mockClear()
    await store.setTaskStatus(validator, 'DONE')

    expect(fetchTimeEntries).toHaveBeenCalled()
  })

  it('leaves the sessions alone when the status changes to something other than done', async () => {
    vi.mocked(fetchTimeEntries).mockClear()
    await store.setTaskStatus(validator, 'BLOCKED')

    expect(fetchTimeEntries).not.toHaveBeenCalled()
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
        autoWrapup: false,
        wrapupDirective: null,
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

  /**
   * "Generate wrapup", set once on the task so the console does not retype the directive after
   * every step. Saved from the same pane as the description and carrying the same obligation:
   * the label, the title, the status and the description all go back untouched.
   */
  describe('the wrapup directive', () => {
    it('turns the toggle on with its directive, leaving everything else in place', async () => {
      await store.saveTaskWrapup(validator, true, '  solo il modulo di export  ')

      expect(updateTask).toHaveBeenCalledWith(validator, {
        label: 'report-builder',
        title: 'Report builder',
        status: 'IN_PROGRESS',
        description: null,
        autoWrapup: true,
        wrapupDirective: 'solo il modulo di export',
        projectId: vega
      })
      const saved = store.tasks.find((task) => task.id === validator)
      expect(saved?.autoWrapup).toBe(true)
      expect(saved?.wrapupDirective).toBe('solo il modulo di export')
    })

    /** A blank message is no message, not a message made of spaces. */
    it('stores a blank directive as none', async () => {
      await store.saveTaskWrapup(validator, true, '   ')

      expect(updateTask).toHaveBeenLastCalledWith(
        validator,
        expect.objectContaining({ autoWrapup: true, wrapupDirective: null })
      )
    })

    /** Nothing changed means nothing is sent, the way the description autosave behaves. */
    it('sends nothing when the toggle and the directive already match', async () => {
      await store.saveTaskWrapup(validator, false, '')

      expect(updateTask).not.toHaveBeenCalled()
      expect(store.saveState).toBe('saved')
    })
  })

  /**
   * The question the description and the wrapup could not answer between them. A brief says what
   * the work is and grows as it is redefined; a wrapup says what it became. What is left was
   * being read out of the two by comparing them, which is what this replaces.
   */
  describe('the checklist', () => {
    it('counts the steps ticked since the wrapup was last written', () => {
      store.selectTask(validator)
      expect(store.wrapupMissesSteps).toBe(1)

      // Nothing to be behind of on a task with no wrapup, however much is ticked.
      store.selectTask(retry)
      expect(store.wrapupMissesSteps).toBe(0)
    })

    it('lists the steps of the task in view, in order, and counts what is open', () => {
      store.selectTask(validator)

      expect(store.selectedTaskSteps.map((step) => step.title)).toEqual([
        'Aggregate the rows',
        'Write the tests'
      ])
      expect(store.openStepCount).toBe(1)

      // A task with no checklist reports none rather than the one belonging to the task before.
      store.selectTask(retry)
      expect(store.selectedTaskSteps).toHaveLength(0)
      expect(store.openStepCount).toBe(0)
    })

    /** Ticking a box changes what the navigator says about the task, without reading it back. */
    it('recounts the task row when a step is ticked', async () => {
      store.selectTask(validator)
      await store.toggleStep('s2' as TaskStepId)

      expect(patchStep).toHaveBeenCalledWith('s2', { done: true })
      expect(store.openStepCount).toBe(0)

      const task = store.tasks.find((candidate) => candidate.id === validator)!
      expect([task.stepCount, task.stepsDone]).toEqual([2, 2])
    })

    /**
     * Accepting and reopening are two intents, not one toggle: each asks for the state it wants,
     * so a second click never walks the step back the way a plain toggle did.
     */
    it('accepts a step forward and reopens it back with explicit calls', async () => {
      store.selectTask(validator)

      await store.acceptStep('s2' as TaskStepId)
      expect(patchStep).toHaveBeenLastCalledWith('s2', { done: true })

      await store.acceptStep('s2' as TaskStepId)
      expect(patchStep).toHaveBeenLastCalledWith('s2', { done: true })

      await store.reopenStep('s2' as TaskStepId)
      expect(patchStep).toHaveBeenLastCalledWith('s2', { done: false })
    })

    it('appends a new step to the end of the list it is added to', async () => {
      store.selectTask(validator)
      await store.addStep(validator, 'Wire the endpoint')

      expect(createStep).toHaveBeenCalledWith(validator, 'Wire the endpoint', undefined)
      expect(store.selectedTaskSteps.map((step) => step.title)).toEqual([
        'Aggregate the rows',
        'Write the tests',
        'Wire the endpoint'
      ])
      expect(store.tasks.find((candidate) => candidate.id === validator)!.stepCount).toBe(3)
    })

    /**
     * A move renumbers everything it displaced, so the whole list comes back and replaces the
     * one held for that task. Writing back only the row that moved is how two steps end up
     * claiming one position.
     */
    it('takes the whole reordered list back from a move', async () => {
      store.selectTask(validator)
      await store.moveStep('s2' as TaskStepId, 0)

      expect(store.selectedTaskSteps.map((step) => step.title)).toEqual([
        'Write the tests',
        'Aggregate the rows'
      ])
    })

    it('drops a deleted step and the count that included it', async () => {
      store.selectTask(validator)
      await store.removeStep('s1' as TaskStepId)

      expect(deleteStep).toHaveBeenCalled()
      expect(store.selectedTaskSteps.map((step) => step.title)).toEqual(['Write the tests'])
      const task = store.tasks.find((candidate) => candidate.id === validator)!
      expect([task.stepCount, task.stepsDone]).toEqual([1, 0])
    })

    /** Four surfaces, and the key that took you to one takes you back, the way W and D do. */
    it('toggles its pane, and does nothing at all with no task in view', () => {
      store.selectedTaskId = null
      store.toggleSteps()
      expect(store.paneFocus).toBe('note')

      store.selectTask(validator)
      store.toggleSteps()
      expect(store.paneFocus).toBe('steps')
      store.toggleSteps()
      expect(store.paneFocus).toBe('note')
    })

    /**
     * A change that arrived over the live feed rather than from a call this window made: a
     * session moved a step over MCP. The rule is "replace what you hold for this task", and the
     * row counts follow.
     */
    it('applies a live checklist event and recounts the task row', () => {
      store.selectTask(validator)

      store.applyStepEvent(validator, [
        { ...steps[0]!, state: 'DONE' },
        { ...steps[1]!, state: 'RUNNING', runningAt: '2026-08-12T14:00:00Z' }
      ])

      expect(store.selectedTaskSteps.map((step) => step.state)).toEqual(['DONE', 'RUNNING'])
      expect(store.runningStep?.id).toBe('s2')
      // Only DONE counts as accepted; the running step does not.
      const task = store.tasks.find((candidate) => candidate.id === validator)!
      expect([task.stepCount, task.stepsDone]).toEqual([2, 1])
    })

    /** A live event for another task never disturbs what this window has open. */
    it('leaves other tasks alone when a live event lands', () => {
      store.selectTask(validator)
      const before = store.selectedTaskSteps.length

      store.applyStepEvent(retry, [])

      expect(store.selectedTaskSteps).toHaveLength(before)
    })

    /**
     * A step a session has claimed is finished work waiting to be accepted. The wrapup is behind
     * it from the moment it was claimed, not from a later console tick.
     */
    it('counts a claimed step against the wrapup by when it was claimed', () => {
      store.selectTask(validator)

      store.applyStepEvent(validator, [
        // Claimed after the wrapup's 12:30, so the wrapup is one step behind.
        { ...steps[0]!, state: 'CLAIMED', done: false, claimedAt: '2026-08-12T15:00:00Z' },
        { ...steps[1]! }
      ])

      expect(store.wrapupMissesSteps).toBe(1)
    })
  })
})
