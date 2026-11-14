import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises } from '@vue/test-utils'
import App from '@/App.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { mountWithHeaderChrome } from './support/mountWithHeaderChrome'
import type { Company, Project, Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Wrapup } from '@/features/wrapup/model/wrapup'
import type { DocumentInput } from '@/features/notes/api/documents.api'
import type {
  CompanyId,
  DocumentId,
  ProjectId,
  TaskId,
  TaskStepId,
  WrapupId
} from '@/model/branded'

/**
 * The console as it actually renders, against a stubbed server. This catches the failures the
 * store tests cannot see: a pane that never mounts, a selection that reaches the wrong pane,
 * a keyboard shortcut wired to nothing, a form that sends the wrong field.
 */
const acme = 'c1' as CompanyId
const vega = 'p1' as ProjectId
const validator = 't1' as TaskId
const retry = 't2' as TaskId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, logoImageVersion: null, projectCount: 1, taskCount: 2, updatedAt: '2026-08-12T10:00:00Z' }
]

const beacon = 'p2' as ProjectId

const projects: Project[] = [
  { id: vega, label: 'vega', taskKey: 'PR', title: 'Vega Platform', status: 'ACTIVE', icon: 'folder', iconImageVersion: null, description: null, blueprintMarkdown: null,
    repoFolder: null, autoCommit: false, mainBranch: null, pullRequestFlow: false, companyId: acme, companyName: 'acme', taskCount: 2, anchor: 'project:vega', updatedAt: '2026-08-12T10:00:00Z' },
  { id: beacon, label: 'beacon', taskKey: 'BC', title: 'Beacon', status: 'PAUSED', icon: 'folder', iconImageVersion: null, description: null, blueprintMarkdown: null,
    repoFolder: null, autoCommit: false, mainBranch: null, pullRequestFlow: false, companyId: acme, companyName: 'acme', taskCount: 0, anchor: 'project:beacon', updatedAt: '2026-08-12T10:00:00Z' }
]

const tasks: Task[] = [
  { id: validator, code: 'PR-1', label: 'report-builder', title: 'Report builder', status: 'IN_PROGRESS', description: null, projectId: vega, projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', projectRepoFolder: null, documentCount: 1, stepCount: 2, stepsDone: 1, draftStepCount: 0, hasWrapup: true, reviewState: 'OPEN', reviewActive: false, claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: null, releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null, anchor: 'task:PR-1', updatedAt: '2026-08-12T10:00:00Z' },
  { id: retry, code: 'PR-2', label: 'retry-policy', title: 'Retry policy', status: 'TODO', description: '## Scope\n\nRitenta solo gli errori 5xx, con backoff esponenziale.', projectId: vega, projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', projectRepoFolder: null, documentCount: 1, stepCount: 0, stepsDone: 0, draftStepCount: 0, hasWrapup: false, reviewState: 'OPEN', reviewActive: true, claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: null, releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null, anchor: 'task:PR-2', updatedAt: '2026-08-12T10:00:00Z' }
]

const shared: RekallDocument = {
  id: 'd1' as DocumentId,
  title: 'kmaster14.md',
  kind: 'notes',
  bodyMarkdown: '# Cluster\n\nAccesso via bastion.',
  taskId: validator,
  contextMode: 'FULL',
  anchor: 'note:00000000',
  updatedAt: '2026-08-12T13:00:00Z'
}

const wrapup: Wrapup = {
  id: 'w1' as WrapupId,
  taskId: validator,
  taskCode: 'report-builder',
  taskTitle: 'Report builder',
  projectLabel: 'vega',
  anchor: 'task:PR-1',
  bodyMarkdown: '## Stato\n\nIl builder gira su POST /api/v1/pipelines.',
  writtenBy: 'CLAUDE',
  createdAt: '2026-08-12T11:00:00Z',
  updatedAt: '2026-08-12T12:30:00Z'
}

const saveWrapup = vi.fn(async () => ({
  ...wrapup,
  bodyMarkdown: 'Riscritto a mano.',
  writtenBy: 'HAND' as const,
  updatedAt: '2026-08-12T14:00:00Z'
}))
const deleteWrapup = vi.fn(async () => undefined)

const updateTask = vi.fn(async () => ({ ...tasks[0]!, status: 'DONE' as const }))
const createTask = vi.fn(async () => ({ ...tasks[0]!, id: 't9' as TaskId }))
const deleteTask = vi.fn(async () => undefined)
const updateProject = vi.fn(async () => projects[0]!)

vi.mock('@/features/catalog/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => companies),
  fetchProjects: vi.fn(async () => projects),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  fetchTasks: vi.fn(async () => tasks),
  fetchTags: vi.fn(async () => []),
  createTag: vi.fn(),
  updateTag: vi.fn(),
  deleteTag: vi.fn(),
  createProject: vi.fn(),
  updateProject: (...args: unknown[]) => updateProject(...(args as [])),
  deleteProject: vi.fn(),
  createTask: (...args: unknown[]) => createTask(...(args as [])),
  updateTask: (...args: unknown[]) => updateTask(...(args as [])),
  deleteTask: (...args: unknown[]) => deleteTask(...(args as []))
}))

const createDocument = vi.fn(async (input: DocumentInput) => ({
  id: 'd9' as DocumentId,
  title: input.title,
  kind: input.kind,
  bodyMarkdown: input.bodyMarkdown,
  taskId: input.taskId,
  contextMode: 'FULL' as const,
  anchor: 'note:d9000000',
  updatedAt: '2026-08-12T15:00:00Z'
}))

vi.mock('@/features/notes/api/documents.api', () => ({
  fetchAllDocuments: vi.fn(async () => [shared]),
  createDocument: (...args: unknown[]) => createDocument(...(args as [DocumentInput])),
  // Answers with the saved note, as the server does: an editor's autosave lands in the store.
  updateDocument: vi.fn(async (id: DocumentId, input: DocumentInput) => ({
    ...shared,
    ...input,
    id
  })),
  deleteDocument: vi.fn()
}))

/** The panes measure and search on their own; here they only have to not reach the network. */
vi.mock('@/features/context/api/context.api', () => ({
  fetchContextSize: vi.fn(async () => ({ characters: 3500, estimatedTokens: 1000, parts: [] }))
}))

vi.mock('@/features/search/api/search.api', () => ({
  searchText: vi.fn(async () => [])
}))

vi.mock('@/features/claude/api/claude.api', async (importOriginal) => {
  const actual = (await importOriginal()) as Record<string, unknown>
  return {
    ...actual,
    fetchClaudeUsage: vi.fn(async () => ({
      status: 'UNAUTHENTICATED',
      limits: [],
      fetchedAt: '',
      retryAt: null
    }))
  }
})

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(async () => []),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn()
}))

vi.mock('@xterm/xterm', () => ({
  Terminal: class {
    element: HTMLElement | null = null
    loadAddon = vi.fn()
    open = vi.fn((host: HTMLElement) => {
      this.element = host
    })
    onData = vi.fn(() => ({ dispose: vi.fn() }))
    onResize = vi.fn(() => ({ dispose: vi.fn() }))
    write = vi.fn()
    reset = vi.fn()
    focus = vi.fn()
    dispose = vi.fn()
  }
}))

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: class {
    fit = vi.fn()
    dispose = vi.fn()
  }
}))

vi.mock('@/features/adherence/api/adherence.api', () => ({
  fetchTaskAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, deviationSteps: [], visits: []
  })),
  fetchProjectAdherence: vi.fn(async () => ({
    stepsTotal: 0, stepsDone: 0, stepsClaimed: 0, acceptedFirstTime: 0, sentBack: 0, passesTotal: 0,
    firstTimeShare: null, deviations: 0, mostSentBack: []
  }))
}))
vi.mock('@/features/wrapup/api/wrapup-versions.api', () => ({
  fetchWrapupFreshness: vi.fn(async () => ({ state: 'unknown' }))
}))
vi.mock('@/features/wrapup/api/wrapups.api', () => ({
  fetchWrapups: vi.fn(async () => [wrapup]),
  saveWrapup: (...args: unknown[]) => saveWrapup(...(args as [])),
  deleteWrapup: (...args: unknown[]) => deleteWrapup(...(args as []))
}))

/**
 * A checklist half done, which is the state the pane exists to show: one piece finished, one
 * still open and carrying the detail of what it has to do.
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
    // After the wrapup's timestamp, so the card has a reason to say it is behind.
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
    bodyMarkdown: 'Un caso per settimana vuota e uno per settimana piena.',
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

const patchStep = vi.fn(async (id: TaskStepId, patch: { done?: boolean }) => ({
  ...steps.find((step) => step.id === id)!,
  ...patch
}))

// The console loads the selected project's components for its [[Name]] chips.
vi.mock('@/features/project/api/project-components.api', () => ({ fetchProjectComponents: vi.fn(async () => []) }))

vi.mock('@/features/steps/api/steps.api', () => ({
  fetchSteps: vi.fn(async () => steps),
  createStep: vi.fn(),
  patchStep: (...args: unknown[]) => patchStep(...(args as [TaskStepId, { done?: boolean }])),
  moveStep: vi.fn(),
  deleteStep: vi.fn()
}))

const startTimeEntry = vi.fn(async (taskId: TaskId) => ({
  id: 'te1',
  taskId,
  taskCode: 'report-builder',
  taskTitle: 'Report builder',
  projectLabel: 'vega',
  anchor: 'task:PR-1',
  startedAt: '2026-08-12T15:00:00Z',
  stoppedAt: null,
  createdAt: '2026-08-12T15:00:00Z',
  updatedAt: '2026-08-12T15:00:00Z'
}))

vi.mock('@/features/time/api/time-entries.api', () => ({
  fetchTimeEntries: vi.fn(async () => []),
  startTimeEntry: (...args: unknown[]) => startTimeEntry(...(args as [TaskId])),
  stopTimeEntry: vi.fn(),
  editTimeEntry: vi.fn(),
  deleteTimeEntry: vi.fn()
}))

vi.mock('@/features/queue/api/runQueue.api', () => {
  const idle = {
    state: 'IDLE',
    startAt: null,
    ceilingPercent: null,
    skipPermissions: false,
    model: null,
    effort: null,
    holdUntil: null,
    holdReason: null,
    items: [],
    updatedAt: '2026-09-23T10:00:00Z'
  }
  return {
    fetchRunQueue: vi.fn(async () => idle),
    saveRunQueueSettings: vi.fn(async () => idle),
    enqueueTask: vi.fn(),
    dequeueItem: vi.fn(),
    moveQueueItem: vi.fn(),
    clearSettledItems: vi.fn(),
    startRunQueue: vi.fn(),
    stopRunQueue: vi.fn()
  }
})

vi.mock('@/features/skills/api/skill-assignments.api', () => ({
  fetchSkillAssignments: vi.fn(async () => ({ taskSkills: [], tagSkills: [] })),
  setTaskSkills: vi.fn(),
  setStepSkills: vi.fn(),
  setTagSkills: vi.fn()
}))

vi.mock('@/features/commits/api/commitReference.api', () => ({
  fetchCommitReferences: vi.fn(async () => []),
  recordLatestCommit: vi.fn()
}))

// Each console follows the shared router; one left mounted would answer the next test's moves.
const mounted: Array<{ unmount(): void }> = []

afterEach(() => {
  mounted.splice(0).forEach((wrapper) => wrapper.unmount())
  vi.unstubAllGlobals()
})

async function mountConsole(path = '/console') {
  setActivePinia(createPinia())
  await router.push(path)
  // App itself no longer loads the catalog on mount — that moved to Bootstrap, the one place
  // guaranteed to run regardless of which route is entered — so the test stands in for it.
  await useConsoleStore().load()
  const wrapper = mountWithHeaderChrome(App, { global: { plugins: [router] } })
  mounted.push(wrapper)
  await flushPromises()
  return wrapper
}

const mountWork = (): ReturnType<typeof mountConsole> => mountConsole('/')

describe('the console', () => {
  beforeEach(() => {
    updateTask.mockClear()
    createTask.mockClear()
    createDocument.mockClear()
    deleteTask.mockClear()
    updateProject.mockClear()
    saveWrapup.mockClear()
    deleteWrapup.mockClear()
    startTimeEntry.mockClear()
    patchStep.mockClear()
    document.body.innerHTML = ''
  })

  it('renders the Work screen with the project scope above the ledger', async () => {
    const wrapper = await mountWork()

    expect(wrapper.find('[data-testid="anchor-trigger"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="work-page"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('All work')
  })

  /**
   * Four levels in one control. A select per level would put three controls above the list and
   * make choosing a project a two-step act; this shows the whole tree and takes one click.
   */
  it('shows companies with their projects nested in one picker', async () => {
    const wrapper = await mountWork()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')

    expect(wrapper.findAll('[data-testid="scope-company"]')).toHaveLength(1)
    expect(wrapper.findAll('[data-testid="scope-project"]')).toHaveLength(2)
    expect(wrapper.find('[data-testid="scope-company"]').text()).toContain('acme')
    const projectRows = wrapper.findAll('[data-testid="scope-project"]')
    expect(projectRows.map((row) => row.text())).toEqual([
      expect.stringContaining('Beacon'),
      expect.stringContaining('Vega Platform')
    ])
    expect(projectRows[1]?.text()).toContain('project:vega')
  })

  it('narrows to a project and says so in the breadcrumb', async () => {
    const wrapper = await mountWork()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')
    await wrapper.findAll('[data-testid="scope-project"]')[1]?.trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="scope-trigger"]').text()).toContain('Vega Platform')
    expect(wrapper.find('[data-testid="scope-trigger"]').text()).toContain('project:vega')
    useWorkStore().view = 'open'
    await flushPromises()
    expect(wrapper.findAll('[data-testid="work-row"]')).toHaveLength(2)
  })

  it('offers creating a company and a project where projects are chosen', async () => {
    const wrapper = await mountWork()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')

    expect(wrapper.find('[data-testid="new-company"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="new-project"]').text()).toContain('acme')

    await wrapper.find('[data-testid="new-company"]').trigger('click')
    expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(true)
    // A company has one name and no anchor label of its own.
    expect(wrapper.find('[data-testid="record-label"]').exists()).toBe(false)
  })

  /** Shortcuts must be inert while writing, or typing a note would navigate away from it. */
  it('ignores shortcuts while a field has focus', async () => {
    await mountConsole()
    const input = document.createElement('input')
    document.body.appendChild(input)
    input.focus()

    input.dispatchEvent(new KeyboardEvent('keydown', { key: '4', bubbles: true }))
    await flushPromises()

    expect(updateTask).not.toHaveBeenCalled()
  })

  // Clicked rather than pressed: earlier tests leave their consoles mounted, and a window-level Q
  // would open a queue panel in every one of them and hold the modal gate for the rest of the file.
  it('carries the run queue beacon in the top bar, which opens the queue', async () => {
    const wrapper = await mountConsole()

    await wrapper.find('[data-testid="queue-beacon"]').trigger('click')
    await flushPromises()

    expect(document.body.querySelector('[data-testid="run-queue-panel"]')).not.toBeNull()
    wrapper.unmount()
  })

  describe('creating, editing and deleting a record', () => {
    /**
     * A task has no label to type: its code is its project's key and the next number, so the
     * anchor is on screen before anything is saved, and nobody saves a record to find out what it became.
     */
    it('shows the code the task will get before anything is saved, with no label to type', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()

      await wrapper.find('[data-testid="record-title"]').setValue('Retry policy, main workflow')
      await flushPromises()

      expect(wrapper.find('[data-testid="record-label"]').exists()).toBe(false)
      expect(wrapper.find('[data-testid="anchor-preview"]').text()).toContain('task:PR-3')
    })

    /**
     * The project is half of the task's anchor, so it is a field and not something the screen
     * decided quietly. It used to be inferred from the scope, which meant a task created from
     * All work landed in whichever project sorted first with nothing on screen saying so.
     */
    it('shows the project a task will land in, and lets it be changed', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()

      const parent = wrapper.find('[data-testid="record-parent"]')
      expect(parent.exists()).toBe(true)
      expect(parent.find('[data-testid="project-picker-title"]').text()).toBe('Vega Platform')
      expect(parent.text()).toContain('project:vega')

      // The code is on screen before a single character is typed.
      expect(wrapper.find('[data-testid="anchor-preview"]').text()).toContain('task:PR-3')
    })

    /** A paused or finished project is still a valid home, so its status sits beside its name rather than hiding it. */
    it('shows the status of the project a task will land in', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()

      expect(wrapper.find('[data-testid="record-parent-status"]').text()).toBe('Active')

      await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')
      const beaconOption = wrapper
        .findAll('[data-testid="project-picker-option"]')
        .find((option) => option.text().includes('project:beacon'))!
      expect(beaconOption.text()).toContain('Paused')
      await beaconOption.trigger('click')
      await flushPromises()

      expect(wrapper.find('[data-testid="record-parent-status"]').text()).toBe('Paused')
    })

    it('creates the task with both names and the project it landed in', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-title"]').setValue('Retry policy')
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(createTask).toHaveBeenCalledWith({
        title: 'Retry policy',
        status: 'TODO',
        description: null,
        projectId: vega,
        tagId: null,
        kind: 'TASK',
        epicId: null
      })
      expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(false)
    })

    it('opens the selected task for editing on E, with its values loaded', async () => {
      const wrapper = await mountConsole(`/task/${validator}`)

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      expect((wrapper.find('[data-testid="record-title"]').element as HTMLInputElement).value)
        .toBe('Report builder')
      expect(wrapper.find('[data-testid="anchor-preview"]').text()).toContain('task:PR-1')
    })

    /** The description has its own pane; the popup neither shows it nor loses it on save. */
    it('edits a task without a description field and saves the description it already has', async () => {
      const wrapper = await mountConsole(`/task/${retry}`)

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(wrapper.find('[data-testid="record-description"]').exists()).toBe(false)
      expect(updateTask).toHaveBeenCalledWith(
        retry,
        expect.objectContaining({ description: '## Scope\n\nRitenta solo gli errori 5xx, con backoff esponenziale.' })
      )
    })

    /**
     * A project's own page now has the room `RecordDialog` never did, so its pencil in the
     * scope picker goes there instead of opening the quick-edit dialog.
     */
    it('sends the project pencil to the project page rather than opening the dialog', async () => {
      const wrapper = await mountWork()

      const pushSpy = vi.spyOn(router, 'push')

      await wrapper.find('[data-testid="scope-trigger"]').trigger('click')
      await wrapper.findAll('[data-testid="edit-project"]')[1]?.trigger('click')

      expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(false)
      expect(pushSpy).toHaveBeenCalledWith(`/projects/${vega}`)
    })

    /** The same field that decides where a task lands is what moves it afterwards. */
    it('moves a task to another project, and gives it that project’s next code', async () => {
      const wrapper = await mountConsole(`/task/${validator}`)

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')
      await wrapper
        .findAll('[data-testid="project-picker-option"]')
        .find((option) => option.text().includes('project:beacon'))!
        .trigger('click')
      await flushPromises()

      expect(wrapper.find('[data-testid="anchor-preview"]').text()).toContain('task:BC-1')

      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(updateTask).toHaveBeenCalledWith(
        validator,
        expect.objectContaining({ projectId: beacon })
      )
    })

    /** Nothing is deleted without the number of things going with it on screen first. */
    it('states the blast radius before deleting, and deletes once confirmed', async () => {
      const wrapper = await mountConsole(`/task/${validator}`)

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      await wrapper.find('[data-testid="record-delete"]').trigger('click')
      await flushPromises()

      expect(wrapper.text()).toContain('1 note unlinked')
      expect(deleteTask).not.toHaveBeenCalled()

      const confirm = wrapper
        .findAll('button')
        .find((button) => button.text() === 'Delete task')!
      await confirm.trigger('click')
      await flushPromises()

      expect(deleteTask).toHaveBeenCalledWith(validator)
    })

    it('refuses to save a record with no title', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(createTask).not.toHaveBeenCalled()
      expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(true)
    })
  })
})
