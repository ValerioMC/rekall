import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchTaskBranch } from '@/features/worktree/api/taskBranch.api'
import type { TaskBranch } from '@/features/worktree/model/taskBranch'
import type { TaskId } from '@/model/branded'

/** The branch of each task the console has shown, read again whenever the task's work moves. */
export const useTaskBranchStore = defineStore('task-branch', () => {
  const byTask = ref<Readonly<Record<string, TaskBranch | null>>>({})

  function of(taskId: TaskId): TaskBranch | null {
    return byTask.value[taskId] ?? null
  }

  function put(taskId: TaskId, branch: TaskBranch | null): void {
    byTask.value = { ...byTask.value, [taskId]: branch }
  }

  async function load(taskId: TaskId): Promise<void> {
    put(taskId, await fetchTaskBranch(taskId))
  }

  return { of, load }
})
