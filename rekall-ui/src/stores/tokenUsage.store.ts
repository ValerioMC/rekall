import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchTokenTallies } from '@/api/token-usage.api'
import { projectTokens, stepTokens, taskTokens, withTaskTallies } from '@/common/token/token-totals'
import { useTokenUsageStream } from '@/composables/useTokenUsageStream'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { Task } from '@/model/catalog'
import type { TokenTally, TokenTotals, TokenUsageStreamEvent } from '@/model/token-usage'

/**
 * The tokens Claude Code sessions spent, per step and per task, read by the shell and kept current
 * by the event stream. A project's total is its tasks' summed here, never stored.
 */
export const useTokenUsageStore = defineStore('tokenUsage', () => {
  const tallies = ref<readonly TokenTally[]>([])

  async function load(): Promise<void> {
    tallies.value = await fetchTokenTallies()
  }

  function apply(event: TokenUsageStreamEvent): void {
    tallies.value = withTaskTallies(tallies.value, event.taskId, event.tallies)
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

  return { tallies, load, apply, ofStep, ofTask, ofProject }
})
