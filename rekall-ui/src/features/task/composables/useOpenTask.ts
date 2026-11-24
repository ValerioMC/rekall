import { useRouter } from 'vue-router'
import type { TaskId } from '@/model/branded'

/** Opens a task as a page, on its Description. */
export function useOpenTask(): (taskId: TaskId) => void {
  const router = useRouter()
  return (taskId) => {
    void router.push({ name: 'task', params: { id: taskId, tab: 'description' } })
  }
}
