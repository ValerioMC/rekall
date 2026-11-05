<script setup lang="ts">
/**
 * Hands the pending entries of the thread to a session. `revise` evaluates the description
 * rewrites and the feedback on the work; `replan` redesigns the steps from the feedback on the
 * plan. With a session already live on the task the line is typed into it; without one a
 * terminal is opened that starts on that line.
 */
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { PlanMode } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

type Applying = Extract<PlanMode, 'REVISE' | 'REPLAN'>

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    anchor: string
    folder: string | null
    mode: 'revise' | 'replan'
    missingHint?: string
  }>(),
  { missingHint: "Set this project's folder on its page to run a session from it" }
)

const store = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const applying = computed<Applying>(() => (props.mode === 'revise' ? 'REVISE' : 'REPLAN'))
const label = computed(() => (props.mode === 'revise' ? 'Run revision' : 'Replan with feedback'))
const pending = computed(() =>
  props.mode === 'revise' ? store.pendingCount('REVISION') + store.pendingCount('IMPLEMENTATION') : store.pendingCount('PLAN')
)
const hasSession = computed(() => Boolean(terminals.terminalForTask(props.taskId)))
const ready = computed(() => hasSession.value || Boolean(props.folder))
const hint = computed(() => {
  if (pending.value === 0) return 'Nothing is pending'
  if (!ready.value) return props.missingHint
  return props.mode === 'revise'
    ? 'Have a session evaluate the pending revisions and feedback on the work'
    : 'Have a session redesign the steps from the pending feedback on the plan'
})

async function apply(): Promise<void> {
  if (pending.value === 0) return
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  const input = { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), kind: preferredLaunchKind() }
  const session = await run(() => terminals.planForTask(props.taskId, props.anchor, input, applying.value))
  if (session === null || session.kind === 'CHAT') return
  store.openTerminal()
}
</script>

<template>
  <AppButton
    size="sm"
    :variant="props.mode === 'revise' ? 'secondary' : 'quiet'"
    :disabled="pending === 0 || isRunning"
    :title="hint"
    :data-testid="`apply-feedback-${props.mode}`"
    @click="apply"
  >
    {{ label }}
    <span
      v-if="pending > 0"
      class="rounded-full bg-accent-soft px-1.5 text-micro font-semibold text-accent"
      data-testid="apply-feedback-count"
    >
      {{ pending }}
    </span>
  </AppButton>
</template>
