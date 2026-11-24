import { useConsoleStore } from '@/stores/console.store'
import { useTaskPlan } from '@/features/steps/composables/useTaskPlan'
import { useTaskReviewActions } from '@/features/task/composables/useTaskReviewActions'
import type { ReviewItem } from '@/features/catalog/model/review'

export interface ReviewDecisions {
  accept: (item: ReviewItem) => Promise<void>
  sendBack: (item: ReviewItem) => Promise<void>
}

/** Accepting or sending back a queue item: a claimed step through the plan, a task handed in whole through its own actions. */
export function useReviewDecisions(): ReviewDecisions {
  const store = useConsoleStore()
  const plan = useTaskPlan()

  function stepOf(item: ReviewItem) {
    return store.steps.find((candidate) => candidate.id === item.stepId)
  }

  function taskOf(item: ReviewItem) {
    return store.tasks.find((candidate) => candidate.id === item.taskId)
  }

  async function accept(item: ReviewItem): Promise<void> {
    const step = stepOf(item)
    const task = taskOf(item)
    if (step) await plan.markForward(step)
    else if (task) await useTaskReviewActions(() => task).accept()
  }

  async function sendBack(item: ReviewItem): Promise<void> {
    const step = stepOf(item)
    const task = taskOf(item)
    if (step) await plan.sendBack(step)
    else if (task) await useTaskReviewActions(() => task).sendBack()
  }

  return { accept, sendBack }
}
