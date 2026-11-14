<script setup lang="ts">
import { computed } from 'vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import type { RevisionRunPhase } from '@/features/feedback/composables/useRevisionRun'

/**
 * The Run revision key, behaving as Run here does: the play mark runs out as the session starts,
 * a ring turns while the server answers, the running seal turns for as long as a pass runs, and
 * the stamped check closes it. Only `idle` takes a press.
 */
const props = defineProps<{ phase: RevisionRunPhase; count: number; hint: string }>()
const emit = defineEmits<{ press: [] }>()

const pressed = computed(() => props.phase === 'working' || props.phase === 'done')
const leaving = computed(() => props.phase === 'launching' || props.phase === 'waiting' || props.phase === 'done')
</script>

<template>
  <button
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 overflow-hidden rounded-key px-2.5 text-small font-medium"
    :class="[pressed ? 'key-lit' : 'key-gold font-semibold', phase !== 'idle' && 'cursor-default']"
    :aria-label="phase === 'working' ? 'Working' : 'Run revision'"
    :aria-busy="phase !== 'idle'"
    :aria-disabled="phase !== 'idle'"
    :title="hint"
    :data-phase="phase"
    data-testid="apply-feedback"
    @click="phase === 'idle' && emit('press')"
  >
    <span class="grid size-3.5 [&>*]:[grid-area:1/1]">
      <svg
        class="size-3.5"
        :class="{ 'run-play-away': leaving, invisible: phase === 'working' }"
        viewBox="0 0 12 12"
        aria-hidden="true"
      >
        <path
          d="M3.4 1.9c0-.6.66-.97 1.17-.65l5.9 3.75c.47.3.47.99 0 1.29l-5.9 3.76c-.51.32-1.17-.05-1.17-.65Z"
          fill="currentColor"
        />
      </svg>
      <span v-if="phase === 'working'" class="run-working block size-4 place-self-center" aria-hidden="true">
        <StepSeal state="RUNNING" />
      </span>
    </span>
    <span class="grid [&>*]:[grid-area:1/1]">
      <span :class="{ 'run-label-away': leaving, invisible: phase === 'working' }">
        Run revision
        <span
          v-if="count > 0"
          class="rounded-key bg-accent-soft px-1.5 text-micro font-semibold text-accent"
          data-testid="apply-feedback-count"
        >
          {{ count }}
        </span>
      </span>
      <span :class="phase === 'working' ? 'run-label-in' : 'invisible'" aria-hidden="true">Working</span>
    </span>

    <span v-if="phase === 'waiting' || phase === 'done'" class="absolute inset-0 grid place-items-center" aria-hidden="true">
      <span v-if="phase === 'done'" class="run-seal block size-[18px]">
        <StepSeal state="DONE" arriving />
      </span>
      <svg v-else class="size-4" viewBox="0 0 16 16" fill="none">
        <circle
          class="run-ring run-ring-open"
          cx="8"
          cy="8"
          r="6.4"
          pathLength="100"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
    </span>

    <span class="sr-only" aria-live="polite">{{ phase === 'done' ? 'Revision finished' : phase === 'working' ? 'Claude is working' : '' }}</span>
  </button>
</template>

<style scoped>
.run-play-away {
  animation: run-play-away 380ms cubic-bezier(0.55, 0, 0.75, 0.2) forwards;
}

.run-label-away {
  animation: run-label-away 180ms ease-out forwards;
}

.run-ring {
  transform-box: fill-box;
  transform-origin: center;
}

.run-ring-open {
  stroke-dasharray: 28 72;
  opacity: 0.8;
  animation: dial-sweep 800ms linear infinite;
}

.run-seal {
  animation: fade-in 140ms ease-out both;
}

.run-seal :deep(.seal[data-state]),
.run-working :deep(.seal[data-state]) {
  filter: drop-shadow(0 1px 1px rgb(0 0 0 / 0.6))
    drop-shadow(0 0 3px color-mix(in srgb, var(--seal) 45%, transparent));
}

.run-working,
.run-label-in {
  animation: fade-in 260ms ease-out both;
}

@keyframes run-play-away {
  0% {
    transform: translateX(0);
    opacity: 1;
  }
  70% {
    opacity: 1;
  }
  100% {
    transform: translateX(72px);
    opacity: 0;
  }
}

@keyframes run-label-away {
  to {
    opacity: 0;
    transform: translateX(6px);
  }
}
</style>
