import { computed, ref, type Ref } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

export interface PlanTask {
  isConfirmingReplan: Ref<boolean>
  missedMessage: Ref<string>
  start: (mode: PlanMode) => void
  confirmReplan: () => Promise<void>
  cancelReplan: () => void
}

/**
 * Has a session plan a task: the line is typed into the one live on the task, or a terminal is
 * opened that starts on it. A replan deletes steps, so it waits for `confirmReplan`, which also
 * carries what the plan missed.
 */
export function usePlanTask(
  taskId: () => TaskId,
  anchor: () => string,
  folder: () => string | null,
  missingHint: string
): PlanTask {
  const console_ = useConsoleStore()
  const terminals = useTerminalStore()
  const toast = useToastStore()
  const { run, isRunning } = useAsyncAction()

  const isConfirmingReplan = ref(false)
  const missedMessage = ref('')
  const ready = computed(() => Boolean(terminals.terminalForTask(taskId())) || Boolean(folder()))

  function start(mode: PlanMode): void {
    if (isRunning.value) return
    if (!ready.value) {
      toast.notifyError(new Error(missingHint))
      return
    }
    if (mode === 'REPLAN') {
      missedMessage.value = ''
      isConfirmingReplan.value = true
      return
    }
    void plan(mode)
  }

  async function confirmReplan(): Promise<void> {
    isConfirmingReplan.value = false
    await plan('REPLAN', missedMessage.value)
  }

  function cancelReplan(): void {
    isConfirmingReplan.value = false
  }

  async function plan(mode: PlanMode, message = ''): Promise<void> {
    const planning = await run(
      () =>
        terminals.planForTask(
          taskId(),
          anchor(),
          { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), kind: preferredLaunchKind() },
          mode,
          message
        ),
      mode === 'REPLAN' ? 'Replanning. The new steps land here.' : 'Planning. Its proposals land here.'
    )
    if (planning === null || planning.kind === 'CHAT') return
    console_.openTerminal()
  }

  return { isConfirmingReplan, missedMessage, start, confirmReplan, cancelReplan }
}
