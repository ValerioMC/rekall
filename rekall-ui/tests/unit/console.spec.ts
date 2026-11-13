import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchTasks, removeCompanyLogoImage, updateProject, uploadCompanyLogoImage } from '@/features/catalog/api/catalog.api'
import { createDocument, fetchAllDocuments, updateDocument } from '@/features/notes/api/documents.api'
import { fetchTimeEntries, startTimeEntry } from '@/features/time/api/time-entries.api'
import { setActivePinia, createPinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskInput } from '@/features/catalog/api/catalog.api'
import type { Company, Project, Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TimeEntry } from '@/features/time/model/time-entry'
import type { Wrapup } from '@/features/wrapup/model/wrapup'
import type {
  CompanyId,
  DocumentId,
  ProjectId,
  TaskId,
  TaskStepId,
  TimeEntryId,
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
  { id: acme, name: 'acme', description: null, logoImageVersion: null, projectCount: 1, taskCount: 2, updatedAt: '2026-08-12T10:00:00Z' },
  { id: globex, name: 'globex', description: null, logoImageVersion: null, projectCount: 1, taskCount: 1, updatedAt: '2026-08-12T10:00:00Z' }
]

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId

const projects: Project[] = [
  { id: vega, label: 'vega', taskKey: 'PR', title: 'Vega Platform', status: 'ACTIVE', icon: 'folder', iconImageVersion: null, description: null, blueprintMarkdown: null,
    repoFolder: null, autoCommit: false, mainBranch: null, pullRequestFlow: false, companyId: acme, companyName: 'acme', taskCount: 2, anchor: 'project:vega', updatedAt: '2026-08-12T10:00:00Z' },
  { id: beacon, label: 'beacon', taskKey: 'PR', title: 'Beacon', status: 'ACTIVE', icon: 'folder', iconImageVersion: null, description: null, blueprintMarkdown: null,
    repoFolder: null, autoCommit: false, mainBranch: null, pullRequestFlow: false, companyId: globex, companyName: 'globex', taskCount: 1, anchor: 'project:beacon', updatedAt: '2026-08-12T10:00:00Z' }
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
  code: label,
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
  stepCount: 0,
  stepsDone: 0, draftStepCount: 0,
  hasWrapup: id === validator,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  graphLaidAt: null,
  releaseSkipped: false,
  kind: 'TASK',
  epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
  anchor: `project:${projectLabel} task:${label}`,
  updatedAt: '2026-08-12T10:00:00Z'
})

const tasks: Task[] = [
  task(validator, 'report-builder', 'Report builder', 'IN_PROGRESS', vega, 'vega', 'Vega Platform', 'acme'),
  task(retry, 'retry-policy', 'Retry policy', 'TODO', vega, 'vega', 'Vega Platform', 'acme'),
  task(wiring, 'wiring', 'Wiring the adapter', 'DONE', beacon, 'beacon', 'Beacon', 'globex')
]

const documents: RekallDocument[] = [
  {
    id: 'd1' as DocumentId,
    title: 'CONTEXT.md',
    kind: 'context',
    bodyMarkdown: 'Il workflow parte da POST /api/v1/pipelines',
    taskId: validator,
    contextMode: 'FULL',
    anchor: 'note:00000000',
    updatedAt: '2026-08-12T12:00:00Z'
  },
  {
    id: 'd2' as DocumentId,
    title: 'kmaster14.md',
    kind: 'notes',
    bodyMarkdown: 'Accesso via bastion',
    taskId: retry,
    contextMode: 'FULL',
    anchor: 'note:00000000',
    updatedAt: '2026-08-12T13:00:00Z'
  },
  {
    id: 'd3' as DocumentId,
    title: 'onboarding.md',
    kind: 'notes',
    bodyMarkdown: 'brew install openjdk',
    taskId: wiring,
    contextMode: 'FULL',
    anchor: 'note:00000000',
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
    taskCode: 'report-builder',
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
    flowX: null,
    flowY: null,
    maxVisits: null,
    startsSession: false,
    connectionId: null,
    passes: [],
    links: [],
    visits: [],
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
    flowX: null,
    flowY: null,
    maxVisits: null,
    startsSession: false,
    connectionId: null,
    passes: [],
    links: [],
    visits: [],
    createdAt: '2026-08-12T10:00:00Z',
    updatedAt: '2026-08-12T10:00:00Z'
  }
]

const createStep = vi.fn(async (taskId: TaskId, title: string) => ({
  id: 's3' as TaskStepId,
  taskId,
  title,
  bodyMarkdown: null,
  state: 'DRAFT' as const,
  done: false,
  runningAt: null,
  claimedAt: null,
  doneAt: null,
  position: 2,
  createdAt: '2026-08-12T16:00:00Z',
  updatedAt: '2026-08-12T16:00:00Z'
}))

/** Mirrors the server: a `done` or `draft` in the patch also settles the step's state. */
const patchStep = vi.fn(async (id: TaskStepId, patch: Partial<TaskStep> & { draft?: boolean }) => {
  const merged = { ...steps.find((step) => step.id === id)!, ...patch }
  if ('done' in patch) merged.state = patch.done ? 'DONE' : 'OPEN'
  if ('draft' in patch) merged.state = patch.draft ? 'DRAFT' : 'OPEN'
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

vi.mock('@/features/catalog/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => companies),
  fetchProjects: vi.fn(async () => projects),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  uploadCompanyLogoImage: vi.fn(),
  removeCompanyLogoImage: vi.fn(),
  fetchTasks: vi.fn(async () => tasks),
  fetchTags: vi.fn(async () => []),
  createTag: vi.fn(),
  updateTag: vi.fn(),
  deleteTag: vi.fn(),
  createProject: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  createTask: vi.fn(),
  updateTask: (...args: Parameters<typeof updateTask>) => updateTask(...args),
  deleteTask: vi.fn()
}))

vi.mock('@/features/notes/api/documents.api', () => ({
  fetchAllDocuments: vi.fn(async () => documents),
  createDocument: vi.fn(),
  updateDocument: vi.fn(),
  deleteDocument: vi.fn()
}))

vi.mock('@/features/wrapup/api/wrapups.api', () => ({
  fetchWrapups: vi.fn(async () => wrapups),
  saveWrapup: (...args: unknown[]) => saveWrapup(...(args as [TaskId, string])),
  deleteWrapup: (...args: unknown[]) => deleteWrapup(...(args as []))
}))

vi.mock('@/features/steps/api/steps.api', () => ({
  fetchSteps: vi.fn(async () => steps),
  createStep: (...args: unknown[]) => createStep(...(args as [TaskId, string])),
  patchStep: (...args: unknown[]) => patchStep(...(args as [TaskStepId, Partial<TaskStep>])),
  moveStep: (...args: unknown[]) => moveStep(...(args as [])),
  deleteStep: (...args: unknown[]) => deleteStep(...(args as []))
}))

const runningEntry = (taskId: TaskId): TimeEntry => ({
  id: `e-${taskId}` as TimeEntryId,
  taskId,
  taskCode: 'report-builder',
  taskTitle: 'Report builder',
  projectLabel: 'vega',
  anchor: 'project:vega task:report-builder',
  startedAt: '2026-08-12T15:00:00Z',
  stoppedAt: null,
  createdAt: '2026-08-12T15:00:00Z',
  updatedAt: '2026-08-12T15:00:00Z'
})

vi.mock('@/features/time/api/time-entries.api', () => ({
  fetchTimeEntries: vi.fn(async () => []),
  startTimeEntry: vi.fn(async (taskId: TaskId) => runningEntry(taskId)),
  stopTimeEntry: vi.fn(),
  editTimeEntry: vi.fn(),
  deleteTimeEntry: vi.fn()
}))

vi.mock('@/features/skills/api/skill-assignments.api', () => ({
  fetchSkillAssignments: vi.fn(async () => ({ taskSkills: [], tagSkills: [] })),
  setTaskSkills: vi.fn(),
  setStepSkills: vi.fn(),
  setTagSkills: vi.fn()
}))

vi.mock('@/features/commits/api/commitReference.api', () => ({
  fetchCommitReferences: vi.fn(async () => []),
  recordLatestCommit: vi.fn(),
  fetchCommitReferenceDiff: vi.fn()
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
    vi.mocked(updateDocument).mockReset()
    vi.mocked(createDocument).mockReset()
    vi.mocked(updateProject).mockReset()
    vi.mocked(startTimeEntry).mockClear()
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
    expect(store.scopeAnchor).toBe('project:vega')

    store.setScope(null)
    expect(store.visibleTasks).toHaveLength(3)
  })

  it('keeps an archived project’s tasks out of the navigator unless the scope names it', () => {
    store.projects = store.projects.map((project) => (project.id === vega ? { ...project, status: 'ARCHIVED' } : project))

    expect(store.visibleTasks.map((t) => t.label)).toEqual(['wiring'])

    store.setScope(acme, vega)
    expect(store.visibleTasks).toHaveLength(2)
  })

  it('lists tasks by title, whatever order they were loaded in', () => {
    const titles = store.visibleTasks.map((task) => task.title)
    expect(titles).toEqual([...titles].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'base' })))
  })

  it("lists a task's notes and only its own", () => {
    store.selectTask(validator)
    expect(store.taskDocuments.map((document) => document.title)).toEqual(['CONTEXT.md'])

    store.selectTask(retry)
    expect(store.taskDocuments.map((document) => document.title)).toEqual(['kmaster14.md'])
  })

  it('opens the first note of a task when the task is picked', () => {
    store.selectTask(retry)
    expect(store.selectedDocId).toBe('d2')
  })

  /** A note has to bring its task along, or the task panes disagree with it. */
  it('follows a note back to a task it belongs to', () => {
    store.selectTask(validator)
    store.selectDocument('d3' as DocumentId)

    expect(store.selectedTaskId).toBe(wiring)
  })

  describe('deleting a note', () => {
    /** Browsing tasks, the editor shows a note on the task in view, so the next one steps in. */
    it('moves the editor to the next note on the task when the open one is deleted', async () => {
      store.selectTask(validator)
      store.selectDocument('d1' as DocumentId)

      await store.deleteNote('d1' as DocumentId)

      expect(store.documents.map((document) => document.id)).not.toContain('d1')
      expect(store.selectedDocId).toBeNull()
      expect(store.selectedTaskId).toBe(validator)
    })
  })

  describe('starting a note', () => {
    beforeEach(() => {
      vi.mocked(createDocument).mockImplementation(async (input) => ({
        id: 'd9' as DocumentId,
        title: input.title,
        kind: input.kind,
        bodyMarkdown: input.bodyMarkdown,
        taskId: input.taskId,
        contextMode: 'FULL',
        anchor: 'note:00000000',
        updatedAt: '2026-08-12T15:00:00Z'
      }))
    })

    it('is born on the task it was given, with its name, and opens in the editor', async () => {
      await store.createNote(validator, 'cluster.md')

      expect(vi.mocked(createDocument).mock.calls[0]?.[0]).toMatchObject({
        title: 'cluster.md',
        kind: 'notes',
        taskId: validator
      })
      expect(store.selectedDocId).toBe('d9')
      expect(store.paneFocus).toBe('note')
    })

    it('falls back to untitled.md when no name is given', async () => {
      await store.createNote(validator, '   ')
      expect(vi.mocked(createDocument).mock.calls[0]?.[0]).toMatchObject({ title: 'untitled.md' })
    })
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
      // CONTEXT.md is 12:00 and the wrapup is 12:30: nothing was written since.
      expect(store.wrapupIsBehind).toBe(0)

      store.documents = [...store.documents, { ...documents[0]!, id: 'd8' as DocumentId, updatedAt: '2026-08-12T13:00:00Z' }]
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
        taskCode: 'retry-policy',
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

      it('lists a task a session created without moving the selection', async () => {
        store.selectTask(validator)
        const created = { ...tasks[0]!, id: 'tCreated' as TaskId, label: 'created', title: 'Created' }
        vi.mocked(fetchTasks).mockResolvedValueOnce([created, ...tasks])

        await store.applyTaskCreated()

        expect(store.tasks.map((task) => task.id)).toContain('tCreated')
        expect(store.selectedTaskId).toBe(validator)
      })

      it('drops a deleted wrapup without moving the pane', () => {
        store.selectTask(validator)
        store.openWrapup()

        store.applyWrapupEvent({ taskId: validator, wrapup: null, deleted: true })

        expect(store.selectedWrapup).toBeNull()
        expect(store.tasks.find((task) => task.id === validator)?.hasWrapup).toBe(false)
        expect(store.paneFocus).toBe('wrapup')
      })

      it('picks up a note a session wrote without moving the selection', async () => {
        store.selectTask(retry)
        const selectedBefore = store.selectedDocId
        const written: RekallDocument = {
          ...documents[0]!,
          id: 'd9' as DocumentId,
          title: 'export-formats.md'
        }
        vi.mocked(fetchAllDocuments).mockResolvedValueOnce([written, ...documents])

        await store.applyNoteWritten()

        expect(store.documents.map((document) => document.id)).toContain('d9')
        expect(store.selectedTaskId).toBe(retry)
        expect(store.selectedDocId).toBe(selectedBefore)
      })
    })
  })

  /**
   * A task row carries a copy of its project's folder, because the button that opens a session
   * lives on the task. Saving the folder has to reach the rows already loaded, or that button
   * goes on saying there is nowhere to open until the window is reloaded.
   */
  it('saves auto-commit through the same full-record update, and keeps what the server decided', async () => {
    vi.mocked(updateProject).mockResolvedValue({ ...projects[0]!, autoCommit: false, mainBranch: null, pullRequestFlow: false })

    await store.saveProjectAutoCommit(vega, true)

    expect(vi.mocked(updateProject).mock.calls[0]?.[1]).toMatchObject({
      autoCommit: true, mainBranch: null, pullRequestFlow: false,
      repoFolder: null,
      label: 'vega'
    })
    // Outside a git repository the server keeps it off, and the store shows what was saved.
    expect(store.projects.find((project) => project.id === vega)?.autoCommit).toBe(false)
  })

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

  /** A status change sends the record back whole: nothing else about it moves. */
  it('keeps the title and the project when only the status changes', async () => {
    await store.setTaskStatus(validator, 'DONE')

    expect(updateTask).toHaveBeenCalledWith(
      validator,
      expect.objectContaining({
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
    it('saves it without moving the title or the status', async () => {
      await store.saveTaskDescription(validator, 'Builds the weekly report from the pipeline runs.')

      expect(updateTask).toHaveBeenCalledWith(validator, {
        title: 'Report builder',
        status: 'IN_PROGRESS',
        description: 'Builds the weekly report from the pipeline runs.',
        projectId: vega,
        tagId: null,
        kind: 'TASK',
        epicId: null
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

    /** Autosave fires on a pause, not on a change, so it lands on text that is already saved. */
    it('sends nothing when the text is what is already stored', async () => {
      await store.saveTaskDescription(validator, '')

      expect(updateTask).not.toHaveBeenCalled()
      expect(store.saveState).toBe('saved')
    })
  })

  describe('the epic a task is in', () => {
    it('files the task under an epic and takes it out again, sending the rest of it unchanged', async () => {
      const epic = 'e1' as TaskId

      await store.setTaskEpic(validator, epic)
      expect(updateTask).toHaveBeenLastCalledWith(validator, expect.objectContaining({ epicId: epic, kind: 'TASK', status: 'IN_PROGRESS' }))
      expect(store.tasks.find((task) => task.id === validator)?.epicId).toBe(epic)

      await store.setTaskEpic(validator, null)
      expect(updateTask).toHaveBeenLastCalledWith(validator, expect.objectContaining({ epicId: null }))
    })

    it('sends nothing when the task is already where it is asked to be', async () => {
      await store.setTaskEpic(validator, null)

      expect(updateTask).not.toHaveBeenCalled()
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

    it('adds a new step as a draft, off the checklist count until it is promoted', async () => {
      store.selectTask(validator)
      await store.addStep(validator, 'Wire the endpoint')

      expect(createStep).toHaveBeenCalledWith(validator, 'Wire the endpoint', undefined)
      expect(store.selectedTaskSteps.map((step) => step.title)).toEqual([
        'Aggregate the rows',
        'Write the tests',
        'Wire the endpoint'
      ])
      const task = store.tasks.find((candidate) => candidate.id === validator)!
      expect([task.stepCount, task.draftStepCount]).toEqual([2, 1])

      await store.promoteStep('s3' as TaskStepId)
      expect(patchStep).toHaveBeenLastCalledWith('s3', { draft: false })
    })

    it('sends an untouched step back to draft', async () => {
      store.selectTask(validator)
      await store.returnStepToDraft('s2' as TaskStepId)

      expect(patchStep).toHaveBeenLastCalledWith('s2', { draft: true })
      expect(store.openStepCount).toBe(0)
      expect(store.tasks.find((candidate) => candidate.id === validator)!.draftStepCount).toBe(1)
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

  /**
   * The timer follows the work instead of the button: writing on a task whose timer is paused
   * starts it, and accepting a claim stops it on the server, which the store then reads back.
   */
  describe('the timer', () => {
    it('starts on a paused task when its description is written', async () => {
      await store.saveTaskDescription(validator, 'Builds the weekly report.')

      expect(startTimeEntry).toHaveBeenCalledWith(validator)
      expect(store.runningEntries.map((entry) => entry.taskId)).toEqual([validator])
    })

    it('asks nothing of the server when the timer is already running', async () => {
      store.timeEntries = [runningEntry(validator)]
      await store.saveTaskDescription(validator, 'Builds the weekly report.')

      expect(startTimeEntry).not.toHaveBeenCalled()
    })

    it('starts on writing the wrapup, adding a step or editing one', async () => {
      await store.saveWrapupBody(retry, 'Retries three times.')
      await store.addStep(wiring, 'Wire the endpoint')
      store.timeEntries = []
      await store.saveStep('s2' as TaskStepId, { title: 'Write the tests', bodyMarkdown: 'All of them.' })

      expect(vi.mocked(startTimeEntry).mock.calls.map(([taskId]) => taskId)).toEqual([retry, wiring, validator])
    })

    /** Ticking, promoting or sending a step back is managing the list, not writing on the task. */
    it('stays paused when a step is only ticked or moved between draft and open', async () => {
      await store.toggleStep('s2' as TaskStepId)
      await store.returnStepToDraft('s2' as TaskStepId)

      expect(startTimeEntry).not.toHaveBeenCalled()
    })

    it('starts on the task in view when a note shared with others is written', async () => {
      vi.mocked(updateDocument).mockImplementation(async (id, input) => ({
        ...documents.find((document) => document.id === id)!,
        ...input
      }) as RekallDocument)
      store.selectTask(retry)
      await store.saveNote('d2' as DocumentId, { bodyMarkdown: 'More on the bastion.' })

      expect(startTimeEntry).toHaveBeenCalledWith(retry)
    })

    it('stays paused when a note is saved with the text it already has', async () => {
      vi.mocked(updateDocument).mockImplementation(async (id, input) => ({
        ...documents.find((document) => document.id === id)!,
        ...input
      }) as RekallDocument)
      store.selectTask(retry)
      const note = documents.find((document) => document.id === 'd2')!
      await store.saveNote('d2' as DocumentId, { title: note.title, bodyMarkdown: note.bodyMarkdown })

      expect(startTimeEntry).not.toHaveBeenCalled()
    })

    it('rereads the sessions once a claimed step is accepted', async () => {
      store.applyStepEvent(validator, [steps[0]!, { ...steps[1]!, state: 'CLAIMED', done: false }])
      store.timeEntries = [runningEntry(validator)]
      vi.mocked(fetchTimeEntries).mockClear()

      await store.acceptStep('s2' as TaskStepId)

      expect(fetchTimeEntries).toHaveBeenCalled()
      expect(store.runningEntries).toHaveLength(0)
    })

    it('leaves the sessions alone when an open step is ticked by hand', async () => {
      store.timeEntries = [runningEntry(validator)]
      vi.mocked(fetchTimeEntries).mockClear()

      await store.toggleStep('s2' as TaskStepId)

      expect(fetchTimeEntries).not.toHaveBeenCalled()
    })
  })

  describe('company logo', () => {
    it('takes the uploaded company as stored, so the picker shows the new version', async () => {
      vi.mocked(uploadCompanyLogoImage).mockResolvedValue({ ...companies[0]!, logoImageVersion: 'v2' })

      await store.setCompanyLogoImage(acme, new File(['x'], 'logo.png', { type: 'image/png' }))

      expect(store.companies.find((company) => company.id === acme)?.logoImageVersion).toBe('v2')
    })

    it('drops the version once the logo is removed', async () => {
      store.companies = store.companies.map((company) => ({ ...company, logoImageVersion: 'v1' }))

      await store.setCompanyLogoImage(acme, null)

      expect(removeCompanyLogoImage).toHaveBeenCalledWith(acme)
      expect(store.companies.find((company) => company.id === acme)?.logoImageVersion).toBeNull()
      expect(store.companies.find((company) => company.id === globex)?.logoImageVersion).toBe('v1')
    })
  })
})
