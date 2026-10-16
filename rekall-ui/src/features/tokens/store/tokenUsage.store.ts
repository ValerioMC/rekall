import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchTokenDays, fetchTokenTallies } from '@/features/tokens/api/token-usage.api'
import { projectTokens, stepTokens, taskTokens, withTaskDays, withTaskTallies } from '@/features/tokens/lib/token-totals'
import { useTokenUsageStream } from '@/features/tokens/composables/useTokenUsageStream'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'
import type { TokenDay, TokenTally, TokenTotals, TokenUsageStreamEvent } from '@/features/tokens/model/token-usage'

/**
 * The tokens Claude Code sessions spent, per step, per task and per task and day, read by the shell
 * and kept current by the event stream. A project's total is its tasks' summed here, never stored.
 */
export const useTokenUsageStore = defineStore('tokenUsage', () => {
  const tallies = ref<readonly TokenTally[]>([])
  const days = ref<readonly TokenDay[]>([])

  async function load(): Promise<void> {
    const [loadedTallies, loadedDays] = await Promise.all([fetchTokenTallies(), fetchTokenDays()])
    tallies.value = loadedTallies
    days.value = loadedDays
  }

  function apply(event: TokenUsageStreamEvent): void {
    tallies.value = withTaskTallies(tallies.value, event.taskId, event.tallies)
    days.value = withTaskDays(days.value, event.taskId, event.days)
  }

  function ofStep(stepId: TaskStepId): TokenTotals {
    return stepTokens(tallies.value, stepId)
  }

  function ofTask(taskId: TaskId): TokenTotals {
    return taskTokens(tallies.value, taskId)
  }

  function ofProject(projectId: ProjectId, tasks: readonly Task[]): TokenTotals {
    return projectTokens(tallies.value, tasks, projectId)
  }

  useTokenUsageStream(apply)

  return { tallies, days, load, apply, ofStep, ofTask, ofProject }
})
