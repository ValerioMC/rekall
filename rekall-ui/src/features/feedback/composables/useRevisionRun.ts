import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { TaskId } from '@/model/branded'

/**
 * `idle` the key takes a press; `launching` and `waiting` the session is on its way; `working` a
 * pass is running; `done` the pass has just been answered, held for a moment before the key returns.
 */
export type RevisionRunPhase = 'idle' | 'launching' | 'waiting' | 'working' | 'done'

type LaunchPhase = 'idle' | 'launching' | 'waiting'

const TRAVEL_MS = 380
const DONE_HOLD_MS = 1200
const MISSING_FOLDER_HINT = "Set this project's folder on its page to run a session from it"

/**
 * Run revision for one task: hands every waiting revision to a session as one pass. With a session
 * already live the `revise` line is typed into it, otherwise a terminal is opened on that line and
 * the view stays on the description. The pass is started once the session is on its way, so a
 * session that cannot start leaves the revisions waiting. The key is offered again only when no
 * pass is running and a revision waits.
 */
export function useRevisionRun(taskId: () => TaskId, anchor: () => string, folder: () => string | null) {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const toast = useToastStore()
  const { run } = useAsyncAction()
  const { waitingCount, runningPass } = storeToRefs(store)

  const launchPhase = ref<LaunchPhase>('idle')
  const finished = ref(false)
  const pendingTimers = new Set<number>()

  const ready = computed(() => Boolean(terminals.terminalForTask(taskId())) || Boolean(folder()))
  const phase = computed<RevisionRunPhase>(() => {
    if (launchPhase.value !== 'idle') return launchPhase.value
    if (runningPass.value !== null) return 'working'
    return finished.value ? 'done' : 'idle'
  })
  const visible = computed(() => waitingCount.value > 0 || phase.value !== 'idle')
  const hint = computed(() => {
    if (runningPass.value !== null) return `Pass ${runningPass.value} is running: revisions written now wait for the next run`
    if (!ready.value) return MISSING_FOLDER_HINT
    return 'Have a session work every waiting revision as one pass'
  })

  watch(runningPass, (now, before) => {
    if (before !== null && now === null) showFinished()
  })

  function showFinished(): void {
    finished.value = true
    void pause(DONE_HOLD_MS).then(() => {
      finished.value = false
    })
  }

  function pause(milliseconds: number): Promise<void> {
    return new Promise((resolve) => {
      const timer = window.setTimeout(() => {
        pendingTimers.delete(timer)
        resolve()
      }, milliseconds)
      pendingTimers.add(timer)
    })
  }

  async function apply(): Promise<void> {
    if (phase.value !== 'idle' || waitingCount.value === 0) return
    if (!ready.value) {
      toast.notifyError(new Error(MISSING_FOLDER_HINT))
      return
    }
    launchPhase.value = 'launching'
    const starting = run(async () => {
      const input = { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), kind: preferredLaunchKind() }
      await terminals.planForTask(taskId(), anchor(), input, 'REVISE')
      await store.startPass(taskId())
    })
    await pause(TRAVEL_MS)
    launchPhase.value = 'waiting'
    await starting
    launchPhase.value = 'idle'
  }

  onBeforeUnmount(() => {
    pendingTimers.forEach((timer) => window.clearTimeout(timer))
    pendingTimers.clear()
  })

  return { phase, visible, hint, waitingCount, runningPass, apply }
}
