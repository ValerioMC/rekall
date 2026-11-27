import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRunProvider } from '@/features/claude/composables/useRunProvider'
import { launchChoices } from '@/common/config/session-launch'
import type { RewriteMode, RewriteStyle } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

/** `idle` takes a press; `launching` the session is on its way; `working` it rewrites; `done` the new text has just arrived. */
export type RewritePhase = 'idle' | 'launching' | 'working' | 'done'

const TRAVEL_MS = 380
const DONE_HOLD_MS = 1200
const POLL_MS = 2000
const GIVE_UP_MS = 10 * 60 * 1000
const MISSING_FOLDER_HINT = "Set this project's folder on its page to run a session from it"

const MODE_OF_STYLE: Record<RewriteStyle, RewriteMode> = { FIX: 'REWRITE_FIX', RESTRUCTURE: 'REWRITE_RESTRUCTURE' }

/**
 * Rewrite one task's description through a session: the `rewrite` line is typed into a live
 * session of the task, or a terminal is opened on it. The session writes from the description, the
 * steps and the decisions of the task, so a task with steps or decisions and no description can be
 * rewritten too: a brief description is written from them. A session that cannot start leaves the
 * description untouched, and one opened for the rewrite is closed once the text is in so the task
 * is released; a session that was already live is left to its owner. The server does not announce the write, so the task is read again until
 * its description changes or its session ends. The text it replaced is held in memory only, for
 * Undo: it is dropped once the person edits and with the task's view.
 */
export function useDescriptionRewrite(taskId: () => TaskId, folder: () => string | null) {
  const store = useConsoleStore()
  const terminals = useTerminalStore()
  const provider = useRunProvider(taskId)
  const toast = useToastStore()
  const { run } = useAsyncAction()

  const launching = ref(false)
  const working = ref(false)
  const finished = ref(false)
  const replacedText = ref<string | null>(null)
  const pendingTimers = new Set<number>()
  let disposed = false

  const description = computed(() => store.tasks.find((task) => task.id === taskId())?.description ?? null)
  const hasSteps = computed(() => store.steps.some((step) => step.taskId === taskId()))
  const hasDecisions = computed(() => (store.decisionsByTask[taskId()]?.decisions.length ?? 0) > 0)
  const ready = computed(() => Boolean(terminals.terminalForTask(taskId())) || Boolean(folder()))
  const phase = computed<RewritePhase>(() => {
    if (launching.value) return 'launching'
    if (working.value) return 'working'
    return finished.value ? 'done' : 'idle'
  })
  const blocker = computed<string | null>(() => {
    if (!description.value?.trim() && !hasSteps.value && !hasDecisions.value) return 'Write a description first'
    if (store.saveState !== 'saved') return 'Wait for the description to be saved'
    return null
  })
  const undoable = computed(() => replacedText.value !== null)

  watch(
    () => store.saveState,
    (state) => {
      if (state === 'unsaved') replacedText.value = null
    }
  )

  function pause(milliseconds: number): Promise<void> {
    return new Promise((resolve) => {
      const timer = window.setTimeout(() => {
        pendingTimers.delete(timer)
        resolve()
      }, milliseconds)
      pendingTimers.add(timer)
    })
  }

  async function awaitRewrittenDescription(before: string): Promise<string | null> {
    const giveUpAt = Date.now() + GIVE_UP_MS
    while (!disposed && Date.now() < giveUpAt) {
      await pause(POLL_MS)
      if (disposed) return null
      const fresh = await store.reloadTask(taskId())
      if ((fresh.description ?? '') !== before) return fresh.description
      if (!terminals.terminalForTask(taskId())) return null
    }
    return null
  }

  async function watchForTheNewText(before: string, openedForRewrite: TerminalId | null): Promise<void> {
    working.value = true
    const rewritten = await run(() => awaitRewrittenDescription(before))
    working.value = false
    if (rewritten === null) return
    if (openedForRewrite && terminals.terminalForTask(taskId())?.id === openedForRewrite) {
      await run(() => terminals.close(openedForRewrite))
    }
    replacedText.value = before
    finished.value = true
    await pause(DONE_HOLD_MS)
    finished.value = false
  }

  async function start(style: RewriteStyle): Promise<void> {
    const before = description.value ?? ''
    if (phase.value !== 'idle' || blocker.value !== null) return
    if (!ready.value) {
      toast.notifyError(new Error(MISSING_FOLDER_HINT))
      return
    }
    replacedText.value = null
    const current = terminals.terminalForTask(taskId())
    const mode = MODE_OF_STYLE[style]
    launching.value = true
    const starting = run(() => terminals.planForTask(taskId(), launchChoices(provider.value), mode))
    await pause(TRAVEL_MS)
    const started = await starting
    launching.value = false
    if (started) {
      const openedForRewrite = started.id !== current?.id && started.mode === mode
      await watchForTheNewText(before, openedForRewrite ? started.id : null)
    }
  }

  async function undo(): Promise<void> {
    const text = replacedText.value
    if (text === null) return
    replacedText.value = null
    await run(() => store.saveTaskDescription(taskId(), text))
  }

  onBeforeUnmount(() => {
    disposed = true
    pendingTimers.forEach((timer) => window.clearTimeout(timer))
    pendingTimers.clear()
  })

  return { phase, blocker, undoable, start, undo }
}
