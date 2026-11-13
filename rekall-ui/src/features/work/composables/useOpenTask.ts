import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/** Brings a task into view in the console: the screen that shows a task until the task page exists. */
export function useOpenTask(): (taskId: TaskId) => void {
  const store = useConsoleStore()
  const router = useRouter()
  return (taskId) => {
    store.selectTask(taskId)
    void router.push({ name: 'console' })
  }
}
