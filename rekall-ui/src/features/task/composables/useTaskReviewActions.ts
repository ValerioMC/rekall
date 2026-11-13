import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'
import { useToastStore } from '@/stores/toast.store'
import type { Task } from '@/features/catalog/model/catalog'

export interface TaskReviewActions {
  accept: () => Promise<void>
  sendBack: () => Promise<void>
}

/** Accepting or sending back a task a session handed in whole. Accept offers Reopen; sending back has no undo. */
export function useTaskReviewActions(task: () => Task): TaskReviewActions {
  const store = useConsoleStore()
  const toast = useToastStore()
  const sendBackGuard = useSendBackGuardStore()
  const { run } = useAsyncAction()

  async function accept(): Promise<void> {
    const target = task()
    const outcome = await run(() => store.acceptTask(target.id))
    if (outcome === null) return
    toast.notify(`Accepted “${target.title}”`, {
      label: 'Reopen',
      run: () => void run(() => store.sendBackTask(target.id))
    })
  }

  function sendBack(): Promise<void> {
    return sendBackGuard.guard(`“${task().title}”`, async () => {
      const outcome = await run(() => store.sendBackTask(task().id))
      if (outcome === null) return
      toast.notify('Sent back to the session')
    })
  }

  return { accept, sendBack }
}
