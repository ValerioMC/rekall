<script setup lang="ts">
/**
 * Has a session propose this task's checklist, or with `mode` `PLAN_FLOW` its flow, as drafts,
 * which land on the Steps pane's shelf and are nothing until promoted there. With `REPLAN` it has
 * the session redesign the draft and open steps the task already has, after asking first, since
 * those are deleted. With a session already live on the task, the line is typed into it; without
 * one, a terminal is opened that starts on that line instead of the plain `/rk`. Either way the
 * planning terminal comes forward.
 */
import { computed, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
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
    /** How many draft and open steps a replan deletes; only read when `mode` is `REPLAN`. */
    replaceableSteps?: number
    missingHint?: string
  }>(),
  { mode: 'PLAN', replaceableSteps: 0, missingHint: "Set this project's folder on its page to plan from it" }
)

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const isConfirming = ref(false)
const liveTerminal = computed(() => terminals.terminalForTask(props.taskId))
const ready = computed(() => Boolean(liveTerminal.value) || Boolean(props.folder))
const plansAFlow = computed(() => props.mode === 'PLAN_FLOW')
const replans = computed(() => props.mode === 'REPLAN')
const proposal = computed(() =>
  plansAFlow.value ? 'this task’s flow, steps and routes,' : 'this task’s steps'
)

const label = computed(() => {
  if (replans.value) return 'Replan'
  return plansAFlow.value ? 'Plan a flow' : 'Plan here'
})
const testId = computed(() => {
  if (replans.value) return 'replan-here'
  return plansAFlow.value ? 'plan-flow-here' : 'plan-here'
})
const tooltip = computed(() => {
  if (replans.value) {
    return liveTerminal.value
      ? 'Have the session running here redesign this task’s draft and open steps'
      : 'Open a session that redesigns this task’s draft and open steps'
  }
  return liveTerminal.value
    ? `Have the session running here propose ${proposal.value} as drafts`
    : `Open a session that proposes ${proposal.value} as drafts`
})
const confirmBlast = computed(
  () => `${stepsCount(props.replaceableSteps)} deleted · not kept anywhere · the new plan lands as drafts`
)

function stepsCount(count: number): string {
  return `${count} ${count === 1 ? 'step' : 'steps'}`
}

function start(): void {
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  if (replans.value) {
    isConfirming.value = true
    return
  }
  void plan()
}

async function confirmReplan(): Promise<void> {
  isConfirming.value = false
  await plan()
}

async function plan(): Promise<void> {
  const planning = await run(
    () =>
      terminals.planForTask(
        props.taskId,
        props.anchor,
        { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort() },
        props.mode
      ),
    replans.value ? 'Replanning. The new steps land here as drafts.' : 'Planning. Its proposals land here as drafts.'
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
    :data-testid="testId"
    :title="tooltip"
    @click="start"
  >
    <svg
      v-if="replans"
      class="size-3 shrink-0"
      :class="isRunning && 'motion-safe:animate-spin'"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9" />
      <path d="M13.5 2.5v3h-3" />
    </svg>
    <span
      v-else
      class="session-caret shrink-0"
      :class="isRunning && 'session-caret-busy'"
      aria-hidden="true"
    />
    {{ label }}
  </button>

  <Transition name="dialog">
    <AppConfirm
      v-if="isConfirming"
      title="Replan this task?"
      body="A session redesigns the steps. Its draft and open steps are replaced; running, claimed and done steps stay."
      :blast="confirmBlast"
      confirm-label="Replan"
      @cancel="isConfirming = false"
      @confirm="confirmReplan"
    />
  </Transition>
</template>
