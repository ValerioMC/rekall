<script setup lang="ts">
/**
 * Has a session propose this task's checklist, or with `mode` `PLAN_FLOW` its flow, as drafts,
 * which land on the Steps pane's shelf and are nothing until promoted there. With a session already
 * live on the task, the plan line is typed into it; without one, a terminal is opened that starts on
 * that line instead of the plain `/rk`. Either way the planning terminal comes forward.
 */
import { computed } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    anchor: string
    folder: string | null
    mode?: PlanMode
    missingHint?: string
  }>(),
  { mode: 'PLAN', missingHint: "Set this project's folder on its page to plan from it" }
)

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const liveTerminal = computed(() => terminals.terminalForTask(props.taskId))
const ready = computed(() => Boolean(liveTerminal.value) || Boolean(props.folder))
const plansAFlow = computed(() => props.mode === 'PLAN_FLOW')
const proposal = computed(() =>
  plansAFlow.value ? 'this task’s flow, steps and routes,' : 'this task’s steps'
)

const tooltip = computed(() =>
  liveTerminal.value
    ? `Have the session running here propose ${proposal.value} as drafts`
    : `Open a session that proposes ${proposal.value} as drafts`
)

async function plan(): Promise<void> {
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  const planning = await run(
    () =>
      terminals.planForTask(
        props.taskId,
        props.anchor,
        { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort() },
        props.mode
      ),
    'Planning. Its proposals land here as drafts.'
  )
  if (planning === null) return
  console_.openTerminal()
}
</script>

<template>
  <button
    type="button"
    class="focus-ring inline-flex h-7 shrink-0 items-center gap-2 rounded-key px-2.5 text-small font-medium disabled:cursor-not-allowed disabled:opacity-60"
    :class="
      ready
        ? 'key-quiet'
        : 'border border-transparent text-text-subtle transition-colors hover:bg-surface-raised hover:text-text-muted'
    "
    :disabled="isRunning"
    :data-testid="plansAFlow ? 'plan-flow-here' : 'plan-here'"
    :title="tooltip"
    @click="plan"
  >
    <span
      class="session-caret shrink-0"
      :class="isRunning && 'session-caret-busy'"
      aria-hidden="true"
    />
    {{ plansAFlow ? 'Plan a flow' : 'Plan here' }}
  </button>
</template>
