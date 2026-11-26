import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import TaskChangesTab from '@/features/task/components/TaskChangesTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import { fetchTaskChangeFiles, fetchTaskChangesIndex } from '@/features/commits/api/taskChanges.api'
import { ApiError } from '@/api/client'
import { changeSources, sourcesInScope } from '@/features/commits/lib/change-sources'
import { PROJECT_REPOSITORY_ID, type ChangeScope } from '@/features/commits/model/changeScope'
import type { Task } from '@/features/catalog/model/catalog'
import type {
  ChangedFile,
  ChangeFileHunks,
  ChangeFileKey,
  DiffHunk,
  DiffLine,
  RepositoryChanges,
  TaskChangesIndex
} from '@/features/commits/model/taskChanges'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/features/commits/api/taskChanges.api', () => ({ fetchTaskChangesIndex: vi.fn(), fetchTaskChangeFiles: vi.fn() }))

const taskId = 't1' as TaskId
const stepId = 's1' as TaskStepId
const snapshotId = 'ffeeddcc-0000-4000-8000-000000000001'
const EMPTY_INDEX: TaskChangesIndex = { snapshotId, repositories: [] }
const patches = new WeakMap<ChangedFile, readonly DiffHunk[]>()

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
  const metadata: ChangedFile = { path, oldPath: null, kind: 'MODIFIED', binary: false, additions, deletions }
  patches.set(metadata, [{ oldStart: 1, oldLines: 1, newStart: 1, newLines: 1, context: '', lines: [line('REMOVED', 1, null, `old ${text}`), line('ADDED', null, 1, `new ${text}`)] }])
  return metadata
}

const PROJECT_COMMITS: RepositoryChanges['commits'] = [
  {
    referenceId: 'c1',
    commitHash: 'aaa1111222',
    subject: 'first change',
    stepId,
    loggedAt: '2026-10-01T09:00:00Z',
    available: true,
    truncated: false,
    files: [file('src/dock.ts', 1, 1, 'dock'), file('src/float.ts', 1, 1, 'float')]
  },
  {
    referenceId: 'c2',
    commitHash: 'bbb2222333',
    subject: 'second change',
    stepId: null,
    loggedAt: '2026-10-01T10:00:00Z',
    available: true,
    truncated: false,
    files: [file('src/dock.ts', 1, 1, 'dock again')]
  }
]

const CHANGES: TaskChangesIndex = {
  snapshotId,
  repositories: [
    {
      componentId: null,
      name: 'vega',
      commits: PROJECT_COMMITS,
      workingTree: { truncated: false, files: [file('notes.md', 1, 0, 'notes')] }
    }
  ]
}

const SITE_ID = '6b2f1c3a-0000-4000-8000-000000000001'
const TWO_REPOSITORIES: TaskChangesIndex = {
  snapshotId: 'ffeeddcc-0000-4000-8000-000000000002',
  repositories: [
    ...CHANGES.repositories,
    {
      componentId: SITE_ID,
      name: 'site',
      commits: [
        {
          referenceId: 'c3',
          commitHash: 'ccc3333444',
          subject: 'site change',
          stepId: null,
          loggedAt: '2026-10-01T11:00:00Z',
          available: true,
          truncated: false,
          files: [file('src/dock.ts', 1, 1, 'site dock')]
        }
      ],
      workingTree: null
    }
  ]
}

const LARGE_KEYS: readonly ChangeFileKey[] = Array.from({ length: 25 }, (_, index) => ({
  componentId: index < 16 ? null : SITE_ID,
  path: `file-${String(index + 1).padStart(2, '0')}.txt`
}))

function largeIndex(snapshotId: string, patchLabel: string): TaskChangesIndex {
  const files = LARGE_KEYS.map(({ path }) => file(path, 1, 1, `${patchLabel} ${path}`))
  return {
    snapshotId,
    repositories: [
      { componentId: null, name: 'vega', workingTree: null, commits: [{ ...PROJECT_COMMITS[0]!, files: files.slice(0, 16) }] },
      {
        componentId: SITE_ID, name: 'site', workingTree: null,
        commits: [
          { ...PROJECT_COMMITS[0]!, referenceId: 'c3', files: files.slice(16).filter((entry) => entry.path !== 'file-21.txt') },
          { ...PROJECT_COMMITS[1]!, referenceId: 'c4', subject: 'Jump target', files: [files[20]!] }
        ]
      }
    ]
  }
}

const LARGE_INDEX = largeIndex('ffeeddcc-0000-4000-8000-000000000003', 'original')
const REFRESHED_INDEX = largeIndex('ffeeddcc-0000-4000-8000-000000000004', 'refreshed')
const INDEXES = [CHANGES, TWO_REPOSITORIES, LARGE_INDEX, REFRESHED_INDEX]

function responseFrom(index: TaskChangesIndex, scope: ChangeScope, keys: readonly ChangeFileKey[]): ChangeFileHunks[] {
  return sourcesInScope(changeSources(index), scope).flatMap((source) => source.files.flatMap((changed) => {
    const componentId = source.repository.id === PROJECT_REPOSITORY_ID ? null : source.repository.id
    const key = keys.find((wanted) => wanted.componentId === componentId && wanted.path === changed.path)
    if (!key) return []
    return [{
      file: key,
      source: source.kind === 'commit' ? { kind: 'commit', referenceId: source.id } : { kind: 'working-tree' },
      hunks: patches.get(changed) ?? []
    }]
  }))
}

function renderedFileIds(): (string | undefined)[] {
  return tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
}

function idsOf(keys: readonly ChangeFileKey[]): string[] {
  return keys.map(({ componentId, path }) => `${componentId ?? PROJECT_REPOSITORY_ID}::${path}`)
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((complete) => { resolve = complete })
  return { promise, resolve }
}

function scrollToEnd(): Promise<void> {
  const column = tab.get('[data-testid="changes-column"]')
  Object.defineProperties(column.element, { scrollHeight: { value: 3500, configurable: true }, clientHeight: { value: 500, configurable: true } })
  column.element.scrollTop = 3000
  return column.trigger('scroll')
}

let pinia: Pinia
let router: Router
let tab: VueWrapper

function mountTab(contextIds: readonly string[] = []): VueWrapper {
  const store = useConsoleStore()
  store.commitReferences = ['c1', 'c2'].map((id) => ({
    id,
    taskId,
    stepId: null,
    stepTitle: null,
    componentId: null,
    componentName: null,
    commitHash: `${id}hash0000`,
    comment: `commit ${id}`,
    inContext: contextIds.includes(id),
    createdAt: '2026-10-01T09:00:00Z'
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
    vi.mocked(fetchTaskChangesIndex).mockReset()
    vi.mocked(fetchTaskChangeFiles).mockReset().mockImplementation(async (_task, snapshot, scope, keys) => {
      const index = INDEXES.find((fixture) => fixture.snapshotId === snapshot)
      if (!index) throw new Error('No fixture for snapshot ' + snapshot)
      return responseFrom(index, scope, keys)
    })
    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' } })
  })

  afterEach(() => {
    tab?.unmount()
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  it('shows every changed file of the task as one diff, with the commit that made each hunk', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)

    mountTab()
    await flushPromises()

    expect(
      tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
    ).toEqual(['project::notes.md', 'project::src/dock.ts', 'project::src/float.ts'])
    const dockHunks = tab.find('[data-file-id="project::src/dock.ts"]').findAll('[data-testid="hunk-origin"]')
    expect(dockHunks.map((origin) => origin.text())).toEqual([
      expect.stringContaining('aaa1111'),
      expect.stringContaining('bbb2222')
    ])
    expect(tab.find('[data-testid="change-totals"]').text()).toContain('3 files')
  })

  it('reads one commit on its own when its chip in a hunk is pressed, and names the other commits that touched the file', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab
      .find('[data-file-id="project::src/dock.ts"]')
      .findAll('[data-testid="hunk-origin"]')[1]!
      .trigger('click')
    await flushPromises()

    expect(
      tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
    ).toEqual(['project::src/dock.ts'])
    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('second change')
    expect(tab.find('[data-testid="file-other-change"]').text()).toContain('earlier by aaa1111')
    expect(tab.find('[data-testid="hunk-origin"]').exists()).toBe(false)
  })

  it('narrows to the commits of one step and keeps every file in the tree', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
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
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
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
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' }, query: { commit: 'c2' } })
    mountTab()
    await flushPromises()

    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('second change')

    await tab.find('[data-testid="scope-all"]').trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.query.commit).toBeUndefined()
  })

  it('moves to another commit when the route names one while the tab is open', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' }, query: { commit: 'c1' } })
    await flushPromises()

    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('first change')
  })

  it('reads the working tree as its own change', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-testid="scope-working-tree"]').trigger('click')
    await flushPromises()

    expect(
      tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
    ).toEqual(['project::notes.md'])
    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('Working tree')
  })

  it('folds a file marked viewed and counts it, without marking anything done', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()

    await tab.find('[data-file-id="project::src/float.ts"] [data-testid="file-viewed"]').trigger('click')

    expect(tab.find('[data-file-id="project::src/float.ts"] [data-testid="file-viewed-note"]').exists()).toBe(
      true
    )
    expect(tab.find('[data-file-id="project::src/float.ts"] [data-testid="file-hunk"]').exists()).toBe(false)
    expect(tab.find('[data-testid="viewed-count"]').text()).toBe('1 of 3 viewed')
  })

  it('switches between split and unified with the u key', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()
    expect(tab.find('[data-testid="diff-hunk-rows"]').attributes('data-layout')).toBe('split')

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'u' }))
    await flushPromises()

    expect(tab.find('[data-testid="diff-hunk-rows"]').attributes('data-layout')).toBe('unified')
  })

  it('shows no repository heading while the task changed one repository', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)

    mountTab()
    await flushPromises()

    expect(tab.find('[data-testid="change-repository-band"]').exists()).toBe(false)
    expect(tab.find('[data-testid="scope-repository"]').exists()).toBe(false)
    expect(tab.find('[data-testid="change-tree-repository-name"]').exists()).toBe(false)
  })

  it('groups the files by repository, keeping a path both repositories changed apart', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(TWO_REPOSITORIES)

    mountTab()
    await flushPromises()

    expect(tab.findAll('[data-testid="change-repository-band"]').map((band) => band.text())).toEqual([
      expect.stringContaining('vega'),
      expect.stringContaining('site')
    ])
    expect(
      tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
    ).toEqual([
      'project::notes.md',
      'project::src/dock.ts',
      'project::src/float.ts',
      `${SITE_ID}::src/dock.ts`
    ])
    expect(tab.findAll('[data-testid="change-tree-repository-name"]').map((name) => name.text())).toEqual([
      expect.stringContaining('vega'),
      expect.stringContaining('site')
    ])
    expect(tab.find('[data-testid="change-totals"]').text()).toContain('2 repositories')
  })

  it('reads one repository on its own from the track', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(TWO_REPOSITORIES)
    mountTab()
    await flushPromises()

    await tab.findAll('[data-testid="scope-repository"]')[1]!.trigger('click')
    await flushPromises()

    expect(
      tab.findAll('[data-testid="changed-file"]').map((section) => section.attributes('data-file-id'))
    ).toEqual([`${SITE_ID}::src/dock.ts`])
    expect(tab.find('[data-testid="change-scope-header"]').text()).toContain('site')
  })

  it('shows the full tree and totals before the first batch settles', async () => {
    const first = deferred<readonly ChangeFileHunks[]>()
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    vi.mocked(fetchTaskChangeFiles).mockReturnValueOnce(first.promise)
    mountTab()
    await flushPromises()

    expect(tab.findAll('[data-testid="change-file-row"]')).toHaveLength(25)
    expect(tab.get('[data-testid="change-totals"]').text()).toContain('25 files')
    expect(renderedFileIds()).toEqual([])
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(1)
    expect(fetchTaskChangeFiles).toHaveBeenCalledWith(taskId, LARGE_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(0, 8))

    first.resolve(responseFrom(LARGE_INDEX, { kind: 'all' }, LARGE_KEYS.slice(0, 8)))
    await flushPromises()
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 8)))
  })

  it('coalesces scroll requests, supports Show more, and stops at the last file', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    mountTab()
    await flushPromises()

    const second = deferred<readonly ChangeFileHunks[]>()
    vi.mocked(fetchTaskChangeFiles).mockReturnValueOnce(second.promise)
    await scrollToEnd()
    await scrollToEnd()
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(2)
    expect(fetchTaskChangeFiles).toHaveBeenLastCalledWith(taskId, LARGE_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(8, 16))
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 8)))

    second.resolve(responseFrom(LARGE_INDEX, { kind: 'all' }, LARGE_KEYS.slice(8, 16)))
    await flushPromises()
    await tab.get('[data-testid="changes-show-more"] button').trigger('click')
    await flushPromises()
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 24)))
    expect(tab.get('[data-testid="changes-show-more"] button').text()).toBe('Show 1 more file')

    await scrollToEnd()
    await flushPromises()
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS))
    expect(tab.find('[data-testid="changes-show-more"]').exists()).toBe(false)
    await scrollToEnd()
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(4)
  })

  it('jumps to file 21 with one request for its batch', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    mountTab()
    await flushPromises()
    const column = tab.get('[data-testid="changes-column"]')
    const scroll = vi.spyOn(column.element, 'scrollTo')

    const target = tab.get('[data-testid="change-file-row"][title="file-21.txt"]')
    await target.trigger('click')
    await flushPromises()

    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(2)
    expect(fetchTaskChangeFiles).toHaveBeenLastCalledWith(taskId, LARGE_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(16, 24))
    expect(renderedFileIds()).toEqual(idsOf([...LARGE_KEYS.slice(0, 8), ...LARGE_KEYS.slice(16, 24)]))
    expect(target.attributes('aria-selected')).toBe('true')
    expect(scroll).toHaveBeenCalledOnce()
    await column.trigger('scroll')
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(2)

    await scrollToEnd()
    await flushPromises()
    expect(fetchTaskChangeFiles).toHaveBeenLastCalledWith(taskId, LARGE_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(24))
    await scrollToEnd()
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(3)
    await tab.get('[data-testid="changes-show-more"] button').trigger('click')
    await flushPromises()
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS))
  })

  it('fetches the next window when the file shortcut crosses a batch boundary', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    mountTab()
    await flushPromises()
    await tab.get('[data-testid="change-file-row"][title="file-08.txt"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'j' }))
    await flushPromises()

    expect(fetchTaskChangeFiles).toHaveBeenLastCalledWith(taskId, LARGE_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(8, 16))
    expect(tab.get('[data-testid="change-file-row"][title="file-09.txt"]').attributes('aria-selected')).toBe('true')
  })

  it('opens a commit deep link outside the first All changes batch directly', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    await router.push({ name: 'task', params: { id: taskId, tab: 'changes' }, query: { commit: 'c4' } })
    mountTab()
    await flushPromises()

    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(1)
    expect(fetchTaskChangeFiles).toHaveBeenCalledWith(taskId, LARGE_INDEX.snapshotId,
      { kind: 'commit', referenceId: 'c4' }, LARGE_KEYS.slice(20, 21))
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(20, 21)))
    expect(tab.get('[data-testid="change-scope-header"]').text()).toContain('Jump target')
  })

  it('retries a failed append without replacing the files or resetting the scroll', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(LARGE_INDEX)
    mountTab()
    await flushPromises()
    const firstFile = tab.get('[data-testid="changed-file"]').element
    vi.mocked(fetchTaskChangeFiles).mockRejectedValueOnce(new Error('Diff request failed'))

    await scrollToEnd()
    await flushPromises()
    const failed = tab.get('[data-testid="changes-batch-failed"]')
    expect(failed.text()).toContain('Diff request failed')
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 8)))
    await scrollToEnd()
    expect(fetchTaskChangeFiles).toHaveBeenCalledTimes(2)

    await failed.get('button').trigger('click')
    await flushPromises()
    expect(tab.get('[data-testid="changed-file"]').element).toBe(firstFile)
    expect(tab.get('[data-testid="changes-column"]').element.scrollTop).toBe(3000)
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 16)))
    expect(tab.find('[data-testid="changes-batch-failed"]').exists()).toBe(false)
  })

  it('drops a late batch when the scope changes', async () => {
    const previous = deferred<readonly ChangeFileHunks[]>()
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    vi.mocked(fetchTaskChangeFiles).mockReturnValueOnce(previous.promise)
    mountTab()
    await flushPromises()
    await tab.get('[data-testid="scope-working-tree"]').trigger('click')
    await flushPromises()

    previous.resolve(responseFrom(CHANGES, { kind: 'all' }, [{ componentId: null, path: 'src/dock.ts' }]))
    await flushPromises()
    expect(renderedFileIds()).toEqual(['project::notes.md'])
    expect(fetchTaskChangeFiles).toHaveBeenLastCalledWith(taskId, CHANGES.snapshotId,
      { kind: 'working-tree', repositoryId: 'project' }, [{ componentId: null, path: 'notes.md' }])
  })

  it.each(['Reload', 'task switch'])('ignores an older batch after %s', async (change) => {
    const previous = deferred<readonly ChangeFileHunks[]>()
    vi.mocked(fetchTaskChangesIndex).mockResolvedValueOnce(LARGE_INDEX).mockResolvedValueOnce(REFRESHED_INDEX)
    vi.mocked(fetchTaskChangeFiles).mockReturnValueOnce(previous.promise)
    mountTab()
    await flushPromises()

    if (change === 'Reload') await tab.get('[data-testid="changes-reload"]').trigger('click')
    else await tab.setProps({ task: { ...makeTask(), id: 't2' as TaskId } })
    await flushPromises()
    previous.resolve(responseFrom(LARGE_INDEX, { kind: 'all' }, LARGE_KEYS.slice(0, 8)))
    await flushPromises()

    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 8)))
    expect(tab.get('[data-testid="changed-file"]').text()).toContain('refreshed file-01.txt')
  })

  it('refreshes an expired snapshot and reissues the needed batch without dropping loaded files', async () => {
    const fresh = deferred<TaskChangesIndex>()
    vi.mocked(fetchTaskChangesIndex).mockResolvedValueOnce(LARGE_INDEX).mockReturnValueOnce(fresh.promise)
    mountTab()
    await flushPromises()
    vi.mocked(fetchTaskChangeFiles).mockRejectedValueOnce(new ApiError(409, 'The Changes snapshot has expired', ''))
    await scrollToEnd()
    await flushPromises()

    expect(fetchTaskChangesIndex).toHaveBeenCalledTimes(2)
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 8)))
    fresh.resolve(REFRESHED_INDEX)
    await flushPromises()

    expect(fetchTaskChangeFiles).toHaveBeenCalledWith(taskId, REFRESHED_INDEX.snapshotId, { kind: 'all' }, LARGE_KEYS.slice(8, 16))
    expect(renderedFileIds()).toEqual(idsOf(LARGE_KEYS.slice(0, 16)))
    expect(tab.get('[data-testid="changed-file"]').text()).toContain('refreshed file-01.txt')
  })

  it('refreshes the index when a logged commit is added or removed', async () => {
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(CHANGES)
    mountTab()
    await flushPromises()
    const store = useConsoleStore()
    const reference = store.commitReferences[0]!

    store.commitReferences.push({ ...reference, id: 'c3' })
    await flushPromises()
    expect(fetchTaskChangesIndex).toHaveBeenCalledTimes(2)
    store.commitReferences.pop()
    await flushPromises()
    expect(fetchTaskChangesIndex).toHaveBeenCalledTimes(3)
  })

  it('shows a fast empty response without showing a loading message', async () => {
    vi.useFakeTimers()
    vi.mocked(fetchTaskChangesIndex).mockResolvedValue(EMPTY_INDEX)

    mountTab()
    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(false)
    expect(tab.get('[data-testid="changes-loading"]').text()).toBe('')
    await flushPromises()
    await vi.advanceTimersByTimeAsync(200)

    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(true)
    expect(tab.find('[data-testid="changes-loading"]').exists()).toBe(false)
    expect(fetchTaskChangeFiles).not.toHaveBeenCalled()
  })

  it('shows a small loading message only while a slow read is pending', async () => {
    vi.useFakeTimers()
    const pending = deferred<TaskChangesIndex>()
    vi.mocked(fetchTaskChangesIndex).mockReturnValue(pending.promise)

    mountTab()
    await vi.advanceTimersByTimeAsync(199)
    expect(tab.get('[data-testid="changes-loading"]').text()).toBe('')
    await vi.advanceTimersByTimeAsync(1)
    expect(tab.get('[data-testid="changes-loading"]').text()).toBe('Loading changes…')
    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(false)

    pending.resolve(EMPTY_INDEX)
    await flushPromises()

    expect(tab.find('[data-testid="changes-loading"]').exists()).toBe(false)
    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(true)
  })

  it('starts a fresh delay for a new task and drops the old result', async () => {
    vi.useFakeTimers()
    const first = deferred<TaskChangesIndex>()
    const second = deferred<TaskChangesIndex>()
    vi.mocked(fetchTaskChangesIndex).mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)

    mountTab()
    await vi.advanceTimersByTimeAsync(150)
    await tab.setProps({ task: { ...makeTask(), id: 't2' as TaskId } })
    await vi.advanceTimersByTimeAsync(50)
    first.resolve(EMPTY_INDEX)
    await flushPromises()

    expect(tab.get('[data-testid="changes-loading"]').text()).toBe('')
    expect(tab.find('[data-testid="changes-empty"]').exists()).toBe(false)
    second.resolve(CHANGES)
    await flushPromises()
    expect(tab.find('[data-testid="changed-file"]').exists()).toBe(true)
  })

  it('shows why the changes could not be read and reads them again on request', async () => {
    vi.useFakeTimers()
    const retry = deferred<TaskChangesIndex>()
    vi.mocked(fetchTaskChangesIndex).mockRejectedValueOnce(new Error('git took too long')).mockReturnValueOnce(retry.promise)

    mountTab()
    await flushPromises()
    expect(tab.find('[data-testid="changes-failed"]').text()).toContain('git took too long')

    await tab.find('[data-testid="changes-failed"] button').trigger('click')
    expect(tab.get('[data-testid="changes-loading"]').text()).toBe('')
    retry.resolve(CHANGES)
    await flushPromises()

    expect(tab.find('[data-testid="changed-file"]').exists()).toBe(true)
  })
})
