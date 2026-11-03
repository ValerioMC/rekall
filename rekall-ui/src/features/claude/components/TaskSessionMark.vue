<script setup lang="ts">
import type { TaskSessionState } from '@/features/claude/model/task-session'

/**
 * The session's own mark, under the task mark in a task row. Three bars: they play while a session
 * is open on the task, stand still and dim once it has ended, and give way to a pulsing amber dot
 * when a session waits for an approval. A task no session was opened on draws nothing.
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
    <template v-else>
      <rect class="session-bar" x="2" y="2" width="2.6" height="6" rx="1.3" />
      <rect class="session-bar" x="5.7" y="2" width="2.6" height="6" rx="1.3" />
      <rect class="session-bar" x="9.4" y="2" width="2.6" height="6" rx="1.3" />
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

.session-bar {
  fill: var(--color-safe);
  transform-box: fill-box;
  transform-origin: center bottom;
}

.session-mark[data-state='LIVE'] .session-bar {
  animation: session-play 1.1s ease-in-out infinite;
}

.session-mark[data-state='LIVE'] .session-bar:nth-child(2) {
  animation-delay: -0.35s;
}

.session-mark[data-state='LIVE'] .session-bar:nth-child(3) {
  animation-delay: -0.7s;
}

.session-mark[data-state='ENDED'] .session-bar {
  fill: var(--color-text-subtle);
  opacity: 0.55;
  transform: scaleY(0.55);
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

@keyframes session-play {
  0%,
  100% {
    transform: scaleY(0.4);
  }
  50% {
    transform: scaleY(1);
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
