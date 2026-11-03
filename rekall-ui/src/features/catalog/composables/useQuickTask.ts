import { computed, type ComputedRef } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { activeQuickTask } from '@/features/catalog/lib/quick-task'
import type { ProjectId } from '@/model/branded'

export interface QuickTaskLauncher {
  readonly busy: ComputedRef<boolean>
  readonly working: ComputedRef<boolean>
  start: (text: string) => Promise<boolean>
}

/** Starts a quick task on a project with the session preferences "Run here" uses, and shows its session. */
export function useQuickTask(projectId: ComputedRef<ProjectId>): QuickTaskLauncher {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const openSession = useOpenSession()
  const { run, isRunning } = useAsyncAction()

  const working = computed(() => activeQuickTask(store.tasks, projectId.value) !== null)

  async function start(text: string): Promise<boolean> {
    const started = await run(() =>
      store.createQuickTask(projectId.value, {
        text,
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort(),
        kind: preferredLaunchKind()
      })
    )
    if (started === null) return false
    terminals.adoptOpened(started.terminal)
    openSession(started.terminal)
    return true
  }

  return { busy: computed(() => isRunning.value), working, start }
}
