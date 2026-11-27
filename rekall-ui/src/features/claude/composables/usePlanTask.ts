import { computed, ref, type Ref } from 'vue'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useOpenSessionInPane } from '@/features/claude/composables/useOpenSessionInPane'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { launchChoices } from '@/common/config/session-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import { useRunProvider } from '@/features/claude/composables/useRunProvider'
import type { TaskId } from '@/model/branded'

export interface PlanTask {
  isRunning: Ref<boolean>
  isConfirmingReplan: Ref<boolean>
  missedMessage: Ref<string>
  start: (mode: PlanMode) => void
  confirmReplan: () => Promise<void>
  cancelReplan: () => void
}

/**
 * Plans or discusses a task in its session, then floats a terminal or shows a chat full-size. A
 * replan deletes steps, so it waits for `confirmReplan`.
 */
export function usePlanTask(
  taskId: () => TaskId,
  folder: () => string | null,
  missingHint: string
): PlanTask {
  const terminals = useTerminalStore()
  const toast = useToastStore()
  const openInPane = useOpenSessionInPane()
  const { run, isRunning } = useAsyncAction()

  const isConfirmingReplan = ref(false)
  const missedMessage = ref('')
  const provider = useRunProvider(taskId)
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
    void plan(mode, '')
  }

  async function confirmReplan(): Promise<void> {
    if (!isConfirmingReplan.value || isRunning.value) return
    isConfirmingReplan.value = false
    await plan('REPLAN', missedMessage.value)
  }

  function cancelReplan(): void {
    isConfirmingReplan.value = false
  }

  async function plan(mode: PlanMode, message: string): Promise<void> {
    const opened = await run(
      () => terminals.planForTask(taskId(), launchChoices(provider.value), mode, message),
      startedMessage(mode)
    )
    if (opened?.kind === 'TERMINAL') terminals.float(opened.id)
    else if (opened) openInPane(opened)
  }

  function startedMessage(mode: PlanMode): string {
    if (mode === 'REPLAN') return 'Replanning. The new steps land here.'
    if (mode === 'REFINE') return 'Refining. Its decisions land here.'
    if (mode === 'DISCUSS') return 'Discussing. What you agree lands on the task.'
    return 'Planning. Its proposals land here.'
  }

  return { isRunning, isConfirmingReplan, missedMessage, start, confirmReplan, cancelReplan }
}
