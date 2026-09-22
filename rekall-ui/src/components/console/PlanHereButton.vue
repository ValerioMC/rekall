<script setup lang="ts">
/**
 * Types `/rk <anchor> plan` into the session already running on this task, so it proposes the
 * checklist as drafts. Shown only while that session is live: without one there is nothing to type
 * into, and opening a fresh terminal would load the task and start working it instead of planning.
 */
import { computed } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { rkPlanCommand } from '@/common/format/rk-command'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ taskId: TaskId; anchor: string }>()

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const { run, isRunning } = useAsyncAction()

const liveTerminal = computed(() => terminals.terminalForTask(props.taskId))

async function plan(): Promise<void> {
  const terminal = liveTerminal.value
  if (!terminal) return
  const sent = await run(
    () => terminals.sendCommand(terminal.id, rkPlanCommand(props.anchor)),
    'Sent to the terminal. Its proposals land here as drafts.'
  )
  if (sent === null) return
  console_.openTerminal()
  terminals.select(terminal.id)
}
</script>

<template>
  <button
    v-if="liveTerminal"
    type="button"
    class="focus-ring inline-flex h-7 shrink-0 items-center gap-2 rounded-[var(--radius-control)] border border-border px-2.5 text-xs font-medium text-text-muted transition-colors hover:border-accent hover:bg-accent-soft hover:text-accent disabled:cursor-not-allowed disabled:opacity-60"
    :disabled="isRunning"
    data-testid="plan-here"
    title="Have the session running here propose this task's steps as drafts"
    @click="plan"
  >
    <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
    Plan here
  </button>
</template>
