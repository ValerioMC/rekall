import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import TaskChangesTab from '@/features/task/components/TaskChangesTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import { fetchTaskChanges } from '@/features/commits/api/taskChanges.api'
import type { Task } from '@/features/catalog/model/catalog'
import type { ChangedFile, DiffLine, TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/features/commits/api/taskChanges.api', () => ({ fetchTaskChanges: vi.fn() }))

const taskId = 't1' as TaskId
const stepId = 's1' as TaskStepId

function makeTask(): Task {
  return {
    id: taskId, code: 'RK-1', label: 'one', title: 'Task one', status: 'IN_PROGRESS', description: null,
    projectId: 'p1' as ProjectId, projectLabel: 'vega', projectTitle: 'Vega', companyName: 'Acme', projectRepoFolder: '/work/vega',
    documentCount: 0, stepCount: 1, stepsDone: 0, draftStepCount: 0, hasWrapup: false, reviewState: 'OPEN', reviewActive: false,
    claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: null,
    releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null,
    trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null,
    pullRequestFailure: null, pullRequestMergedAt: null, anchor: 'task:RK-1', updatedAt: '2026-10-01T10:00:00Z'
  }
}

function makeStep(): TaskStep {
  return {
    id: stepId, taskId, title: 'Float the terminal', bodyMarkdown: null, state: 'DONE', done: true, runningAt: null,
    claimedAt: null, doneAt: '2026-10-01T08:00:00Z', position: 1, flowX: null, flowY: null, maxVisits: null,
    startsSession: false, connectionId: null, passes: [], links: [], visits: [], createdAt: '2026-10-01T07:00:00Z', updatedAt: '2026-10-01T07:00:00Z'
  }
}

function line(kind: DiffLine['kind'], oldNumber: number | null, newNumber: number | null, text: string): DiffLine {
  return { kind, oldNumber, newNumber, text }
}

function file(path: string, additions: number, deletions: number, text: string): ChangedFile {
  return {
    path, oldPath: null, kind: 'MODIFIED', binary: false, additions, deletions,
    hunks: [{ oldStart: 1, oldLines: 1, newStart: 1, newLines: 1, context: '', lines: [line('REMOVED', 1, null, `old ${text}`), line('ADDED', null, 1, `new ${text}`)] }]
  }
}

const CHANGES: TaskChanges = {
  commits: [
    { referenceId: 'c1', commitHash: 'aaa1111222', subject: 'first change', stepId, loggedAt: '2026-10-01T09:00:00Z', available: true, truncated: false, files: [file('src/dock.ts', 1, 1, 'dock'), file('src/float.ts', 1, 1, 'float')] },
    { referenceId: 'c2', commitHash: 'bbb2222333', subject: 'second change', stepId: null, loggedAt: '2026-10-01T10:00:00Z', available: true, truncated: false, files: [file('src/dock.ts', 1, 1, 'dock again')] }
  ],
  workingTree: { truncated: false, files: [file('notes.md', 1, 0, 'notes')] }
}

let pinia: Pinia
let router: Router
let tab: VueWrapper

function mountTab(contextIds: readonly string[] = []): VueWrapper {
  const store = useConsoleStore()
  store.commitReferences = ['c1', 'c2'].map((id) => ({
    id, taskId, stepId: null, stepTitle: null, commitHash: `${id}hash0000`, comment: `commit ${id}`, inContext: contextIds.includes(id), createdAt: '2026-10-01T09:00:00Z'
  }))
  store.tasks = [makeTask()]
  store.selectedTaskId = taskId
  store.steps = [makeStep()]
  store.isLoading = false
  tab = mount(TaskChangesTab, { props: { task: makeTask() }, attachTo: document.body, global: { plugins: [pinia, router] } })
  return tab
}

describe('TaskChangesTab', () => {
  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)
    router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/task/:id/:tab?', name: 'task', component: { template: '<div />' } }] })
    vi.mocked(fetchTaskChanges).mockReset()
    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' } })
  })

  afterEach(() => {
    tab?.unmount()
    document.body.innerHTML = ''
  })

  it('shows every changed file of the task as one diff, with the commit that made each hunk', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)

    mountTab()
    await flushPromises()

    expect(tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-path'))).toEqual(['notes.md', 'src/dock.ts', 'src/float.ts'])
    const dockHunks = tab.find('[data-file-path="src/dock.ts"]').findAll('[data-testid="hunk-origin"]')
    expect(dockHunks.map((origin) => origin.text())).toEqual([expect.stringContaining('aaa1111'), expect.stringContaining('bbb2222')])
    expect(tab.find('[data-testid="change-totals"]').text()).toContain('3 files')
  })

  it('reads one commit on its own when its chip in a hunk is pressed, and names the other commits that touched the file', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-file-path="src/dock.ts"]').findAll('[data-testid="hunk-origin"]')[1]!.trigger('click')
    await flushPromises()

    expect(tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-path'))).toEqual(['src/dock.ts'])
    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('second change')
    expect(tab.find('[data-testid="file-other-change"]').text()).toContain('earlier by aaa1111')
    expect(tab.find('[data-testid="hunk-origin"]').exists()).toBe(false)
  })

  it('narrows to the commits of one step and keeps every file in the tree', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-testid="scope-step"]').trigger('click')
    await flushPromises()

    expect(tab.findAll('[data-testid="changed-file"]')).toHaveLength(2)
    const rows = tab.findAll('[data-testid="change-file-row"]')
    expect(rows).toHaveLength(3)
    expect(rows.filter((row) => row.attributes('data-in-scope') === 'false').map((row) => row.text())).toEqual([expect.stringContaining('notes.md')])
  })

  it('adds the commit being read to the /rk context from its header, and marks it on the track', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab(['c2'])
    await flushPromises()
    expect(tab.find('[data-testid="change-context-control"]').exists()).toBe(false)
    expect(tab.findAll('[data-testid="scope-commit-in-context"]')).toHaveLength(1)

    const store = useConsoleStore()
    store.setCommitReferenceInContext = vi.fn().mockResolvedValue(undefined)
    await tab.findAll('[data-testid="scope-commit"]')[0]!.trigger('click')
    await flushPromises()
    const control = tab.get('[data-testid="change-context-control"] button')
    expect(control.text()).toBe('Add to /rk context')
    await control.trigger('click')

    expect(store.setCommitReferenceInContext).toHaveBeenCalledWith('c1', true)
  })

  it('opens on the commit named by the route and keeps the route in step with the scope', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' }, query: { commit: 'c2' } })
    mountTab()
    await flushPromises()

    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('second change')

    await tab.find('[data-testid="scope-all"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.query.commit).toBeUndefined()
  })

  it('moves to another commit when the route names one while the tab is open', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' }, query: { commit: 'c1' } })
    await flushPromises()

    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('first change')
  })

  it('reads the working tree as its own change', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-testid="scope-working-tree"]').trigger('click')
    await flushPromises()

    expect(tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-path'))).toEqual(['notes.md'])
    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('Working tree')
  })

  it('folds a file marked viewed and counts it, without marking anything done', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-file-path="src/float.ts"] [data-testid="file-viewed"]').trigger('click')

    expect(tab.find('[data-file-path="src/float.ts"] [data-testid="file-viewed-note"]').exists()).toBe(true)
    expect(tab.find('[data-file-path="src/float.ts"] [data-testid="file-hunk"]').exists()).toBe(false)
    expect(tab.find('[data-testid="viewed-count"]').text()).toBe('1 of 3 viewed')
  })

  it('switches between split and unified with the u key', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()
    expect(tab.find('[data-testid="diff-hunk-rows"]').attributes('data-layout')).toBe('split')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'u' }))
    await flushPromises()

    expect(tab.find('[data-testid="diff-hunk-rows"]').attributes('data-layout')).toBe('unified')
  })

  it('says so when nothing has changed yet', async () => {
    vi.mocked(fetchTaskChanges).mockResolvedValue({ commits: [], workingTree: null })

    mountTab()
    await flushPromises()

    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(true)
  })

  it('shows why the changes could not be read and reads them again on request', async () => {
    vi.mocked(fetchTaskChanges).mockRejectedValueOnce(new Error('git took too long')).mockResolvedValueOnce(CHANGES)

    mountTab()
    await flushPromises()
    expect(tab.find('[data-testid="changes-failed"]').text()).toContain('git took too long')

    await tab.find('[data-testid="changes-failed"] button').trigger('click')
    await flushPromises()

    expect(tab.find('[data-testid="changed-file"]').exists()).toBe(true)
  })
})
