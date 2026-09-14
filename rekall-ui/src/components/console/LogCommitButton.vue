<script setup lang="ts">
import { computed, ref } from 'vue'
import { recordLatestCommit } from '@/api/commitReference.api'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId, TaskStepId } from '@/model/branded'

const props = withDefaults(
  defineProps<{
    taskId: TaskId
    stepId?: TaskStepId | null
    folder: string | null
    /** `header`: the neutral chrome next to Run here / Open in terminal. `bar`: the plain-text
     *  button inside a review bar, next to Accept / Send back. */
    variant?: 'header' | 'bar'
  }>(),
  { stepId: null, variant: 'header' }
)

const store = useConsoleStore()
const toast = useToastStore()

const missing = "Set this project's folder on its page before logging a commit here."
const target = computed(() => (props.stepId ? 'this step' : 'this task'))

const phase = ref<'idle' | 'logging' | 'logged'>('idle')
const lastHash = ref('')
const nudging = ref(false)
let settleTimer: ReturnType<typeof setTimeout> | null = null
let nudgeTimer: ReturnType<typeof setTimeout> | null = null

async function log(): Promise<void> {
  if (phase.value === 'logging') return
  if (!props.folder) {
    toast.notifyError(new Error(missing))
    if (nudgeTimer) clearTimeout(nudgeTimer)
    nudging.value = false
    requestAnimationFrame(() => {
      nudging.value = true
      nudgeTimer = setTimeout(() => (nudging.value = false), 400)
    })
    return
  }
  phase.value = 'logging'
  try {
    const logged = await recordLatestCommit(props.taskId, props.stepId ?? null)
    store.applyCommitReference(logged)
    lastHash.value = logged.commitHash.slice(0, 7)
    toast.notify(`Logged “${logged.comment}”.`)
    phase.value = 'logged'
    if (settleTimer) clearTimeout(settleTimer)
    settleTimer = setTimeout(() => (phase.value = 'idle'), 1900)
  } catch (caught) {
    toast.notifyError(caught)
    phase.value = 'idle'
  }
}
</script>

<template>
  <button
    type="button"
    class="commit-btn focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] px-2.5 text-xs font-medium transition-all duration-150 active:translate-y-px disabled:cursor-not-allowed disabled:opacity-60"
    :class="[
      variant === 'header'
        ? 'border border-border bg-transparent text-text-muted hover:border-border-strong hover:bg-surface-raised hover:text-text'
        : 'border border-border-strong px-3 text-[11.5px] transition-colors hover:border-accent hover:text-accent',
      variant === 'bar' && (phase === 'idle' ? 'text-text-subtle' : 'text-accent'),
      nudging && 'nudge',
      phase === 'logged' && 'flash'
    ]"
    :disabled="phase === 'logging'"
    :aria-label="`Log the latest commit against ${target}`"
    :title="folder ? `Log the tip of ${folder} against ${target}` : missing"
    data-testid="log-commit"
    @click="log"
  >
    <span class="relative grid size-3.5 shrink-0 place-items-center">
      <Transition name="glyph" mode="out-in">
        <span
          v-if="phase === 'logging'"
          key="spin"
          class="size-3 animate-spin rounded-full border-[1.6px] border-current/25 border-t-current"
          aria-hidden="true"
        />
        <svg v-else-if="phase === 'logged'" key="check" class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M2.1 6.3 4.6 8.8 9.9 3.4"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <svg v-else key="node" class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M1 6h2.7M8.3 6H11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          <circle cx="6" cy="6" r="2.15" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </Transition>
    </span>
    <Transition name="glyph" mode="out-in">
      <span v-if="phase === 'logged'" key="hash" class="font-mono text-[11px] tabular-nums">{{ lastHash }}</span>
      <span v-else-if="phase === 'logging'" key="logging">Logging…</span>
      <span v-else key="idle">Log commit</span>
    </Transition>
  </button>
</template>

<style scoped>
.glyph-enter-active,
.glyph-leave-active {
  transition:
    opacity 110ms ease,
    transform 110ms ease;
}
.glyph-enter-from,
.glyph-leave-to {
  opacity: 0;
  transform: scale(0.55);
}

@keyframes commit-nudge {
  0%,
  100% {
    transform: translateX(0);
  }
  25% {
    transform: translateX(-3px);
  }
  75% {
    transform: translateX(3px);
  }
}
.nudge {
  animation: commit-nudge 380ms ease;
}
</style>
