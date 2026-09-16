<script setup lang="ts">
import { computed, ref } from 'vue'
import { recordCommit, recordLatestCommit } from '@/api/commitReference.api'
import CommitPicker from '@/components/console/CommitPicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId, TaskStepId } from '@/model/branded'
import type { CommitReference } from '@/model/commitReference'

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
const pickerOpen = ref(false)
const host = ref<HTMLElement | null>(null)
let settleTimer: ReturnType<typeof setTimeout> | null = null
let nudgeTimer: ReturnType<typeof setTimeout> | null = null

/** What is already logged against exactly this task/step, so the picker can mark those rows. */
const loggedHashes = computed(() =>
  store.commitReferences
    .filter((reference) => reference.taskId === props.taskId && reference.stepId === (props.stepId ?? null))
    .map((reference) => reference.commitHash)
)

/** No folder means nothing to read: say so and shake, instead of opening a picker with nothing in it. */
function refuseWithoutFolder(): boolean {
  if (props.folder) return false
  toast.notifyError(new Error(missing))
  if (nudgeTimer) clearTimeout(nudgeTimer)
  nudging.value = false
  requestAnimationFrame(() => {
    nudging.value = true
    nudgeTimer = setTimeout(() => (nudging.value = false), 400)
  })
  return true
}

async function record(write: () => Promise<CommitReference>): Promise<void> {
  if (phase.value === 'logging') return
  phase.value = 'logging'
  try {
    const logged = await write()
    store.applyCommitReference(logged)
    lastHash.value = logged.commitHash.slice(0, 7)
    toast.notify(`Logged “${logged.comment}”.`)
    phase.value = 'logged'
    pickerOpen.value = false
    if (settleTimer) clearTimeout(settleTimer)
    settleTimer = setTimeout(() => (phase.value = 'idle'), 1900)
  } catch (caught) {
    toast.notifyError(caught)
    phase.value = 'idle'
  }
}

async function log(): Promise<void> {
  if (refuseWithoutFolder()) return
  await record(() => recordLatestCommit(props.taskId, props.stepId ?? null))
}

async function logPicked(hash: string): Promise<void> {
  await record(() => recordCommit(props.taskId, props.stepId ?? null, hash))
}

function togglePicker(): void {
  if (pickerOpen.value) {
    pickerOpen.value = false
    return
  }
  if (refuseWithoutFolder()) return
  pickerOpen.value = true
}
</script>

<template>
  <div
    ref="host"
    class="relative inline-flex h-7 shrink-0 items-stretch"
    :class="nudging && 'nudge'"
    data-testid="log-commit-group"
  >
    <button
      type="button"
      class="commit-btn focus-ring relative inline-flex items-center gap-1.5 rounded-l-[var(--radius-control)] border px-2.5 text-xs font-medium transition-all duration-150 active:translate-y-px disabled:cursor-not-allowed disabled:opacity-60"
      :class="[
        variant === 'header'
          ? 'border-border bg-transparent text-text-muted hover:border-border-strong hover:bg-surface-raised hover:text-text'
          : 'border-border-strong px-3 text-[11.5px] transition-colors hover:border-accent hover:text-accent',
        variant === 'bar' && (phase === 'idle' ? 'text-text-subtle' : 'text-accent'),
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
          <svg
            v-else-if="phase === 'logged'"
            key="check"
            class="size-3.5"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
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
        <span v-if="phase === 'logged'" key="hash" class="font-mono text-[11px] tabular-nums">{{
          lastHash
        }}</span>
        <span v-else-if="phase === 'logging'" key="logging">Logging…</span>
        <span v-else key="idle">Log commit</span>
      </Transition>
    </button>

    <button
      type="button"
      class="focus-ring -ml-px grid w-6 shrink-0 place-items-center rounded-r-[var(--radius-control)] border transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-60"
      :class="[
        variant === 'header'
          ? 'border-border text-text-muted hover:border-border-strong hover:bg-surface-raised hover:text-text'
          : 'border-border-strong text-text-subtle hover:border-accent hover:text-accent',
        pickerOpen &&
          (variant === 'header'
            ? 'border-border-strong bg-surface-raised text-text'
            : 'border-accent text-accent')
      ]"
      :disabled="phase === 'logging'"
      :aria-label="`Pick a commit to log against ${target}`"
      :title="folder ? 'Pick a commit from the recent log, or paste a hash' : missing"
      aria-haspopup="dialog"
      :aria-expanded="pickerOpen"
      data-testid="pick-commit"
      @click="togglePicker"
    >
      <svg
        class="size-3 transition-transform duration-150"
        :class="pickerOpen && 'rotate-180'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M3 4.5 6 7.5l3-3"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <CommitPicker
      v-if="pickerOpen && host"
      :task-id="taskId"
      :anchor="host"
      :logged-hashes="loggedHashes"
      :busy="phase === 'logging'"
      @pick="logPicked"
      @close="pickerOpen = false"
    />
  </div>
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
