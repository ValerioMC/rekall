<script setup lang="ts">
import type { TaskSessionState } from '@/features/claude/model/task-session'

/**
 * The session's own mark, under the task mark in a task row. While a session is open on the task
 * it is the task mark's in-progress face in indigo: same size, an arc orbiting against the task
 * mark's own turn, a breathing dot at the centre. It rests as a dim ring once it has
 * ended, becomes an indigo disc with a check when its work is handed in, and gives way to a pulsing
 * amber dot when a session waits for an approval. A task no session was opened on draws nothing.
 */
defineProps<{ state: Exclude<TaskSessionState, 'NONE'> }>()
</script>

<template>
  <svg
    class="session-mark"
    viewBox="0 0 14 14"
    fill="none"
    :data-state="state"
    data-testid="task-session-mark"
    aria-hidden="true"
  >
    <template v-if="state === 'APPROVAL'">
      <circle class="session-ping" cx="7" cy="7" r="2.2" />
      <circle class="session-dot" cx="7" cy="7" r="2.2" />
    </template>
    <template v-else-if="state === 'CLAIMED'">
      <circle class="session-disc" cx="7" cy="7" r="4" />
      <path class="session-check" d="M5.1 7.1 6.5 8.5 9 5.7" />
    </template>
    <template v-else-if="state === 'LIVE'">
      <circle class="session-orbit" cx="7" cy="7" r="5.9" />
      <circle class="session-arc" cx="7" cy="7" r="5.9" pathLength="100" />
      <circle class="session-core" cx="7" cy="7" r="2" />
    </template>
    <template v-else>
      <circle class="session-track" cx="7" cy="7" r="3.6" />
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

.session-orbit {
  stroke: color-mix(in srgb, var(--color-session) 28%, transparent);
  stroke-width: 1.2;
}

.session-core {
  fill: var(--color-session-strong);
  transform-box: fill-box;
  transform-origin: center;
  animation: dial-core 1.7s ease-in-out infinite;
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
  transform-box: view-box;
  transform-origin: 7px 7px;
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
