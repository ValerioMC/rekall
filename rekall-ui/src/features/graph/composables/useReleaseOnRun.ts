import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId } from '@/model/branded'

/**
 * Lays the project's release graph on a task the console is about to run. A refusal (drafts still
 * waiting, a graph that cannot run) is told and the session starts anyway, without the graph.
 */
export function useReleaseOnRun() {
  const store = useConsoleStore()
  const toast = useToastStore()

  async function releaseBeforeRun(taskId: TaskId): Promise<void> {
    try {
      const laid = await store.releaseOnRun(taskId)
      if (laid) toast.notify(`${laid} laid on the task before it runs.`)
    } catch (caught) {
      toast.notifyError(caught)
    }
  }

  return { releaseBeforeRun }
}
