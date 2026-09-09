<script setup lang="ts">
import { computed, ref } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeStore } from '@/stores/claude.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId, TaskStepId } from '@/model/branded'

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    folder: string | null
    stepId?: TaskStepId | null
    missingHint?: string
  }>(),
  { stepId: null, missingHint: "Set this project's folder on its page to run a session from it" }
)

const console_ = useConsoleStore()
const claude = useClaudeStore()
const toast = useToastStore()
const { run, isRunning } = useAsyncAction()

const ready = computed(() => Boolean(props.folder))

async function launch(): Promise<void> {
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  const created = await run(
    () =>
      claude.startForTask(props.taskId, {
        stepId: props.stepId,
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort()
      }),
    'Session started'
  )
  if (created) console_.openClaude()
}

const previewing = ref(false)
</script>

<template>
  <button
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] border px-2.5 text-xs font-medium transition-all duration-150 active:translate-y-px disabled:cursor-not-allowed disabled:opacity-60"
    :class="
      ready
        ? 'border-accent bg-accent-soft text-accent hover:bg-accent hover:text-accent-ink'
        : 'border-transparent bg-transparent text-text-subtle hover:bg-surface-raised hover:text-text-muted'
    "
    :disabled="isRunning"
    data-testid="launch-claude-session"
    @click="launch"
    @mouseenter="previewing = true"
    @mouseleave="previewing = false"
    @focus="previewing = true"
    @blur="previewing = false"
  >
    <span
      v-if="isRunning"
      class="size-3 animate-spin rounded-full border-[1.6px] border-current/25 border-t-current"
      aria-hidden="true"
    />
    <svg v-else class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <rect x="0.9" y="1.6" width="10.2" height="8.8" rx="1.6" stroke="currentColor" stroke-width="1.1" />
      <path d="M4.6 4.3 7.3 6 4.6 7.7Z" fill="currentColor" />
    </svg>
    <span>Run here</span>
  </button>
</template>
