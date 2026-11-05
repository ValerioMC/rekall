<script setup lang="ts">
/**
 * Run revision: hands every waiting entry of the thread, of any kind, to a session as one pass.
 * With a session already live on the task the `revise` line is typed into it; without one a
 * terminal is opened that starts on that line, and the view stays on the description. The pass is
 * started once the session is on its way, so a session that cannot start leaves the entries waiting.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredLaunchKind, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { TaskId } from '@/model/branded'

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    anchor: string
    folder: string | null
    missingHint?: string
    variant?: 'primary' | 'secondary'
  }>(),
  { missingHint: "Set this project's folder on its page to run a session from it", variant: 'secondary' }
)

const store = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()
const { waitingCount, runningPass } = storeToRefs(store)

const hasSession = computed(() => Boolean(terminals.terminalForTask(props.taskId)))
const ready = computed(() => hasSession.value || Boolean(props.folder))
const passRunning = computed(() => runningPass.value !== null)
const hint = computed(() => {
  if (passRunning.value) return `Pass ${runningPass.value} is running: entries written now wait for the next run`
  if (waitingCount.value === 0) return 'Nothing is waiting for a run'
  if (!ready.value) return props.missingHint
  return 'Have a session work every waiting entry as one pass: revisions, feedback on the work and on the plan'
})

async function apply(): Promise<void> {
  if (waitingCount.value === 0 || passRunning.value) return
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  const input = { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), kind: preferredLaunchKind() }
  await run(async () => {
    await terminals.planForTask(props.taskId, props.anchor, input, 'REVISE')
    await store.startPass(props.taskId)
  })
}
</script>

<template>
  <AppButton
    size="sm"
    :variant="variant"
    :disabled="waitingCount === 0 || passRunning || isRunning"
    :title="hint"
    data-testid="apply-feedback"
    @click="apply"
  >
    Run revision
    <span
      v-if="waitingCount > 0"
      class="rounded-full bg-accent-soft px-1.5 text-micro font-semibold text-accent"
      data-testid="apply-feedback-count"
    >
      {{ waitingCount }}
    </span>
  </AppButton>
</template>
