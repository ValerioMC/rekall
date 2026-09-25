<script setup lang="ts">
/**
 * Has a session propose this task's checklist as drafts, which land on the Steps pane's shelf and
 * are nothing until promoted there. With a session already live on the task, `/rk <anchor> plan` is
 * typed into it; without one, a terminal is opened that starts on that line instead of the plain
 * `/rk`, so it plans the task rather than working it. Either way the planning terminal comes forward.
 */
import { computed } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { TaskId } from '@/model/branded'

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    anchor: string
    folder: string | null
    missingHint?: string
  }>(),
  { missingHint: "Set this project's folder on its page to plan from it" }
)

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const liveTerminal = computed(() => terminals.terminalForTask(props.taskId))
const ready = computed(() => Boolean(liveTerminal.value) || Boolean(props.folder))

const tooltip = computed(() =>
  liveTerminal.value
    ? 'Have the session running here propose this task’s steps as drafts'
    : 'Open a session that proposes this task’s steps as drafts'
)

async function plan(): Promise<void> {
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  const planning = await run(
    () =>
      terminals.planForTask(props.taskId, props.anchor, {
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort()
      }),
    'Planning. Its proposals land here as drafts.'
  )
  if (planning === null) return
  console_.openTerminal()
}
</script>

<template>
  <button
    type="button"
    class="focus-ring inline-flex h-7 shrink-0 items-center gap-2 rounded-[var(--radius-control)] border px-2.5 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-60"
    :class="
      ready
        ? 'border-border text-text-muted hover:border-accent hover:bg-accent-soft hover:text-accent'
        : 'border-transparent text-text-subtle hover:bg-surface-raised hover:text-text-muted'
    "
    :disabled="isRunning"
    data-testid="plan-here"
    :title="tooltip"
    @click="plan"
  >
    <span
      class="session-caret shrink-0"
      :class="isRunning && 'session-caret-busy'"
      aria-hidden="true"
    />
    Plan here
  </button>
</template>
