import { computed, ref, type Ref } from 'vue'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { launchChoices } from '@/common/config/session-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import type { SessionProvider } from '@/features/claude/model/session-provider'
import type { TaskId } from '@/model/branded'

export interface PlanTask {
  isRunning: Ref<boolean>
  isConfirmingReplan: Ref<boolean>
  missedMessage: Ref<string>
  start: (mode: PlanMode, provider?: SessionProvider) => void
  confirmReplan: () => Promise<void>
  cancelReplan: () => void
}

/**
 * Has a session plan or discuss a task: the line is typed into the one live on the task, or a terminal is
 * opened that starts on it. A replan deletes steps, so it waits for `confirmReplan`, which also
 * carries what the plan missed.
 */
export function usePlanTask(
  taskId: () => TaskId,
  folder: () => string | null,
  missingHint: string
): PlanTask {
  const terminals = useTerminalStore()
  const toast = useToastStore()
  const { run, isRunning } = useAsyncAction()

  const isConfirmingReplan = ref(false)
  const missedMessage = ref('')
  const ready = computed(() => Boolean(terminals.terminalForTask(taskId())) || Boolean(folder()))

  function start(mode: PlanMode, provider?: SessionProvider): void {
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
    void plan(mode, '', provider)
  }

  async function confirmReplan(): Promise<void> {
    if (!isConfirmingReplan.value || isRunning.value) return
    isConfirmingReplan.value = false
    await plan('REPLAN', missedMessage.value)
  }

  function cancelReplan(): void {
    isConfirmingReplan.value = false
  }

  async function plan(mode: PlanMode, message: string, provider?: SessionProvider): Promise<void> {
    const opened = await run(
      () => terminals.planForTask(taskId(), launchChoices(provider), mode, message),
      startedMessage(mode)
    )
    if (opened?.kind === 'TERMINAL') terminals.float(opened.id)
  }

  function startedMessage(mode: PlanMode): string {
    if (mode === 'REPLAN') return 'Replanning. The new steps land here.'
    if (mode === 'REFINE') return 'Refining. Its decisions land here.'
    if (mode === 'DISCUSS') return 'Discussing. What you agree lands on the task.'
    return 'Planning. Its proposals land here.'
  }

  return { isRunning, isConfirmingReplan, missedMessage, start, confirmReplan, cancelReplan }
}
