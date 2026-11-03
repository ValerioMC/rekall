import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskBranchPanel from '@/features/worktree/components/TaskBranchPanel.vue'
import * as api from '@/features/worktree/api/taskBranch.api'
import type { TaskBranch } from '@/features/worktree/model/taskBranch'
import type { TaskId } from '@/model/branded'

vi.mock('@/features/worktree/api/taskBranch.api')

const task = 'task-1' as TaskId

function branch(overrides: Partial<TaskBranch> = {}): TaskBranch {
  return {
    branch: 'rekall/report',
    commitsAhead: 2,
    changedFiles: ['src/export.ts', 'src/import.ts'],
    mergesCleanly: true,
    worktreeFolder: '/home/rekall/worktrees/vega/report',
    hasUnsavedChanges: false,
    mergeCommand: 'git merge rekall/report',
    ...overrides
  }
}

async function render(found: TaskBranch | null) {
  vi.mocked(api.fetchTaskBranch).mockResolvedValue(found)
  const wrapper = mount(TaskBranchPanel, { props: { taskId: task, signature: 'a' } })
  await flushPromises()
  return wrapper
}

describe('the Branch panel of a task in review', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('is not drawn for a task with no branch', async () => {
    const wrapper = await render(null)

    expect(wrapper.find('[data-testid="task-branch"]').exists()).toBe(false)
  })

  it('shows the branch, what it holds and that it merges cleanly, in words', async () => {
    const wrapper = await render(branch())

    expect(wrapper.get('[data-testid="branch-name"]').text()).toBe('rekall/report')
    expect(wrapper.get('[data-testid="branch-ahead"]').text()).toBe('2 commits ahead, 2 files changed')
    expect(wrapper.get('[data-testid="branch-merge"]').text()).toContain("Merges cleanly into the project's current branch.")
    expect(wrapper.findAll('[data-testid="branch-files"] li').map((row) => row.text())).toEqual(['src/export.ts', 'src/import.ts'])
  })

  it('says it conflicts, and says when git cannot tell', async () => {
    const conflict = await render(branch({ mergesCleanly: false }))
    expect(conflict.get('[data-testid="branch-merge"]').text()).toContain('Conflicts with')

    const unknown = await render(branch({ mergesCleanly: null }))
    expect(unknown.get('[data-testid="branch-merge"]').text()).toContain('could not tell')
  })

  it('offers the merge command to copy and no button that merges', async () => {
    const copy = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: copy }, configurable: true })
    const wrapper = await render(branch())

    await wrapper.get('[data-testid="branch-copy"]').trigger('click')

    expect(wrapper.get('[data-testid="branch-command"]').text()).toBe('git merge rekall/report')
    expect(copy).toHaveBeenCalledWith('git merge rekall/report')
    expect(wrapper.text().toLowerCase()).not.toContain('merge it')
    expect(wrapper.findAll('button').map((button) => button.text())).toEqual(['Copy merge command', 'Remove worktree'])
  })

  it('removes a clean worktree and keeps showing the branch', async () => {
    vi.mocked(api.removeTaskWorktree).mockResolvedValue(branch({ worktreeFolder: null }))
    const wrapper = await render(branch())

    await wrapper.get('[data-testid="branch-remove-worktree"]').trigger('click')
    await flushPromises()

    expect(api.removeTaskWorktree).toHaveBeenCalledWith(task)
    expect(wrapper.find('[data-testid="branch-remove-worktree"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="branch-worktree"]').text()).toContain('the branch and its commits stay')
  })

  it('says a worktree with unsaved changes is left alone and cannot be removed from here', async () => {
    const wrapper = await render(branch({ hasUnsavedChanges: true }))

    expect(wrapper.get('[data-testid="branch-worktree"]').text()).toContain('unsaved changes')
    expect((wrapper.get('[data-testid="branch-remove-worktree"]').element as HTMLButtonElement).disabled).toBe(true)
  })

  it('reads the branch again when the work moves', async () => {
    const wrapper = await render(branch())

    await wrapper.setProps({ signature: 'b' })
    await flushPromises()

    expect(api.fetchTaskBranch).toHaveBeenCalledTimes(2)
  })
})
