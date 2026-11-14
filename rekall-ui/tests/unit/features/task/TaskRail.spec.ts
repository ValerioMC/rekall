import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskRail from '@/features/task/components/TaskRail.vue'
import { installStorage } from '../../support/memoryStorage'
import { fetchTaskBranch } from '@/features/worktree/api/taskBranch.api'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeResourcesStore } from '@/features/claude-resources/store/claudeResources.store'
import { workRows } from '@/features/work/lib/work-rows'
import type { Task } from '@/features/catalog/model/catalog'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { TaskBranch } from '@/features/worktree/model/taskBranch'
import type { ProjectId, TaskId } from '@/model/branded'

vi.mock('@/features/context/api/context.api', () => ({ fetchContextSize: vi.fn(() => new Promise(() => undefined)) }))
vi.mock('@/features/worktree/api/taskBranch.api', () => ({ fetchTaskBranch: vi.fn() }))
vi.mock('@/features/dependency/api/dependencies.api')

const taskId = 't1' as TaskId

function task(): Task {
  return {
    id: taskId,
    label: 'rail',
    code: 'RK-1',
    title: 'Rail',
    status: 'TODO',
    projectId: 'p1' as ProjectId,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    stepCount: 0,
    stepsDone: 0,
    reviewState: 'OPEN',
    kind: 'TASK',
    anchor: 'task:RK-1',
    updatedAt: '2026-10-01T10:00:00Z'
  } as Task
}

function commit(): CommitReference {
  return {
    id: 'c1',
    taskId,
    stepId: null,
    stepTitle: null,
    componentId: null,
    componentName: null,
    commitHash: 'abc1234def',
    comment: 'Wire up the button',
    inContext: false,
    createdAt: '2026-09-14T10:00:00Z'
  }
}

function branch(): TaskBranch {
  return {
    branch: 'rekall/rail',
    commitsAhead: 1,
    changedFiles: [],
    mergesCleanly: true,
    worktreeFolder: '/work/rail',
    hasUnsavedChanges: false
  }
}

function hidden(element: Element): boolean {
  return (element as HTMLElement).style.display === 'none'
}

async function render() {
  const store = useConsoleStore()
  store.tasks = [task()]
  store.selectTask(taskId)
  const row = workRows([task()], [], () => 0)[0]!
  const wrapper = mount(TaskRail, { props: { row } })
  await flushPromises()
  return wrapper
}

describe('TaskRail', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
    useClaudeResourcesStore().loaded = true
    vi.mocked(fetchTaskBranch).mockReset().mockResolvedValue(null)
  })

  it('groups status and anchor, labels and context, and hides Git while the task has no branch or commit', async () => {
    const wrapper = await render()

    expect(wrapper.find('[data-testid="rail-identity"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="rail-labels"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="rail-context"]').exists()).toBe(true)
    expect(hidden(wrapper.get('[data-testid="rail-git"]').element)).toBe(true)
  })

  it('shows Git with the branch and the commits the task has', async () => {
    vi.mocked(fetchTaskBranch).mockResolvedValue(branch())
    useConsoleStore().commitReferences = [commit()]
    const wrapper = await render()

    const git = wrapper.get('[data-testid="rail-git"]')
    expect(hidden(git.element)).toBe(false)
    expect(git.get('[data-testid="branch-name"]').text()).toBe('rekall/rail')
    expect(git.find('[data-testid="commit-rail"]').exists()).toBe(true)
  })

  it('folds to an icon bar with the choice remembered, and opens again from an icon', async () => {
    const wrapper = await render()

    await wrapper.get('[data-testid="task-rail-toggle"]').trigger('click')

    expect(usePreferencesStore().taskRailCollapsed).toBe(true)
    expect(wrapper.get('[data-testid="task-rail"]').attributes('data-collapsed')).toBe('true')
    expect(wrapper.find('[data-testid="rail-identity"]').exists()).toBe(false)

    await wrapper.get('[data-testid="task-rail-icons"] button').trigger('click')

    expect(usePreferencesStore().taskRailCollapsed).toBe(false)
    expect(wrapper.find('[data-testid="rail-identity"]').exists()).toBe(true)
  })
})
