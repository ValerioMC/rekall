<script setup lang="ts">
/**
 * Has a session propose this task's checklist as drafts, which land on the Steps pane's shelf and
 * are nothing until promoted there. Behind its arrow: the task's flow while it has no steps, and
 * a replan of the draft and open steps it has, after asking first, since those are deleted. With a
 * session already live on the task, the line is typed into it; without one, a terminal is opened
 * that starts on that line instead of the plain `/rk`. Either way the planning terminal comes
 * forward.
 */
import { computed, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppSplitButton, { type SplitButtonOption } from '@/components/ui/AppSplitButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

type PlanAlternative = Exclude<PlanMode, 'PLAN'>

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    anchor: string
    folder: string | null
    /** Whether the task can take a flow: it has no steps and no drafts. */
    offersFlow?: boolean
    /** How many draft and open steps a replan deletes; the replan is offered while it is above zero. */
    replaceableSteps?: number
    missingHint?: string
  }>(),
  { offersFlow: false, replaceableSteps: 0, missingHint: "Set this project's folder on its page to plan from it" }
)

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const isConfirming = ref(false)
const liveTerminal = computed(() => terminals.terminalForTask(props.taskId))
const ready = computed(() => Boolean(liveTerminal.value) || Boolean(props.folder))
const tooltip = computed(() =>
  liveTerminal.value
    ? 'Have the session running here propose this task’s steps as drafts'
    : 'Open a session that proposes this task’s steps as drafts'
)
const alternatives = computed<readonly SplitButtonOption<PlanAlternative>[]>(() => [
  ...(props.offersFlow ? [flowOption()] : []),
  ...(props.replaceableSteps > 0 ? [replanOption()] : [])
])
const confirmBlast = computed(
  () => `${stepsCount(props.replaceableSteps)} deleted · not kept anywhere · the new plan lands as drafts`
)

function flowOption(): SplitButtonOption<PlanAlternative> {
  return {
    id: 'PLAN_FLOW',
    label: 'Plan a flow',
    hint: 'Proposes the steps with the routes between them, as drafts'
  }
}

function replanOption(): SplitButtonOption<PlanAlternative> {
  return {
    id: 'REPLAN',
    label: 'Replan',
    hint: `Redesigns the ${stepsCount(props.replaceableSteps)} still to do; they are deleted`
  }
}

function stepsCount(count: number): string {
  return `${count} ${count === 1 ? 'step' : 'steps'}`
}

function start(mode: PlanMode): void {
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  if (mode === 'REPLAN') {
    isConfirming.value = true
    return
  }
  void plan(mode)
}

async function confirmReplan(): Promise<void> {
  isConfirming.value = false
  await plan('REPLAN')
}

async function plan(mode: PlanMode): Promise<void> {
  const planning = await run(
    () =>
      terminals.planForTask(
        props.taskId,
        props.anchor,
        { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), kind: preferredLaunchKind() },
        mode
      ),
    mode === 'REPLAN' ? 'Replanning. The new steps land here as drafts.' : 'Planning. Its proposals land here as drafts.'
  )
  if (planning === null || planning.kind === 'CHAT') return
  console_.openTerminal()
}
</script>

<template>
  <AppSplitButton
    size="sm"
    variant="quiet"
    :options="alternatives"
    menu-label="Other ways to plan"
    :disabled="isRunning"
    :title="tooltip"
    data-testid="plan-here"
    @click="start('PLAN')"
    @choose="start"
  >
    <span class="session-caret shrink-0" :class="isRunning && 'session-caret-busy'" aria-hidden="true" />
    Plan here
  </AppSplitButton>

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
