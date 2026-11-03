<script setup lang="ts">
import type { TaskSessionState } from '@/features/claude/model/task-session'

/**
 * The session's own mark, under the task mark in a task row. An indigo arc glows and spins against
 * the task mark's own orbit while a session is open on the task, rests as a dim ring once it has
 * ended, becomes an indigo disc with a check when its work is handed in, and gives way to a pulsing
 * amber dot when a session waits for an approval. A task no session was opened on draws nothing.
 */
defineProps<{ state: Exclude<TaskSessionState, 'NONE'> }>()
</script>

<template>
  <svg
    class="session-mark"
    viewBox="0 0 14 10"
    fill="none"
    :data-state="state"
    data-testid="task-session-mark"
    aria-hidden="true"
  >
    <template v-if="state === 'APPROVAL'">
      <circle class="session-ping" cx="7" cy="5" r="2.2" />
      <circle class="session-dot" cx="7" cy="5" r="2.2" />
    </template>
    <template v-else-if="state === 'CLAIMED'">
      <circle class="session-disc" cx="7" cy="5" r="4" />
      <path class="session-check" d="M5.1 5.1 6.5 6.5 9 3.7" />
    </template>
    <template v-else>
      <circle class="session-track" cx="7" cy="5" r="3.6" />
      <circle v-if="state === 'LIVE'" class="session-arc" cx="7" cy="5" r="3.6" pathLength="100" />
    </template>
  </svg>
</template>

<style scoped>
.session-mark {
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.session-track {
  stroke: color-mix(in srgb, var(--color-session) 28%, transparent);
  stroke-width: 1.4;
}

.session-mark[data-state='ENDED'] .session-track {
  stroke: var(--color-text-subtle);
  opacity: 0.55;
}

.session-arc {
  stroke: var(--color-session-strong);
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-dasharray: 30 70;
  filter: drop-shadow(0 0 1.2px var(--color-session));
  transform-box: fill-box;
  transform-origin: center;
  animation: session-spin 2.2s linear infinite;
}

.session-disc {
  fill: var(--color-session);
  filter: drop-shadow(0 0 1.2px var(--color-session));
}

.session-check {
  stroke: var(--color-session-ink);
  stroke-width: 1.3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.session-dot {
  fill: var(--color-warn);
}

.session-ping {
  stroke: var(--color-warn);
  stroke-width: 1;
  transform-box: fill-box;
  transform-origin: center;
  animation: session-ping 1.4s cubic-bezier(0.2, 0.7, 0.3, 1) infinite;
}

@keyframes session-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(-360deg);
  }
}

@keyframes session-ping {
  from {
    opacity: 0.8;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(2.3);
  }
}
</style>
