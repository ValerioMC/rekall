import { ref } from 'vue'
import { defineStore } from 'pinia'
import { preferredProvider, saveTaskProvider, savedTaskProvider } from '@/common/config/session-launch'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskId } from '@/model/branded'

/** The provider a task was switched to, kept in this browser. A task never switched follows the Settings default. */
export const useTaskProviderStore = defineStore('taskProvider', () => {
  const switched = ref(new Map<TaskId, SessionProvider>())

  function providerFor(taskId: TaskId): SessionProvider {
    return switched.value.get(taskId) ?? savedTaskProvider(taskId) ?? preferredProvider()
  }

  function choose(taskId: TaskId, provider: SessionProvider): void {
    switched.value.set(taskId, provider)
    saveTaskProvider(taskId, provider)
  }

  return { providerFor, choose }
})
