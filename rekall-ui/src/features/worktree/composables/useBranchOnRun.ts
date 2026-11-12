import { useTaskBranchStore } from '@/features/worktree/store/taskBranch.store'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId } from '@/model/branded'

/**
 * Puts a task on its own branch before a session opened outside Rekall starts, when its project
 * keeps tasks on branches. A refusal is told and the session does not start on the main branch.
 */
export function useBranchOnRun() {
  const branches = useTaskBranchStore()
  const toast = useToastStore()

  /** The task's worktree, else `folder`; null when the branch could not be made. */
  async function folderToRunIn(taskId: TaskId, folder: string): Promise<string | null> {
    try {
      return (await branches.start(taskId)) ?? folder
    } catch (caught) {
      toast.notifyError(caught)
      return null
    }
  }

  return { folderToRunIn }
}
