<script setup lang="ts">
import { TASK_ACTIVITY_LABEL, type TaskActivity } from '@/features/workingset/model/task-activity'

/**
 * Three bars, never a dot, so it is not read as a second status. Moving: a session works.
 * Still and lit: work was executed. Still and short: nothing happened. The bars keep their
 * heights under reduced motion, so the state reads without it.
 */
defineProps<{ activity: TaskActivity }>()

const BAR_COLOR: Readonly<Record<TaskActivity, string>> = {
  WORKING: 'bg-session-strong',
  EXECUTED: 'bg-session',
  INACTIVE: 'bg-border-strong'
}
</script>

<template>
  <span
    class="activity-mark"
    :class="{ 'activity-mark-working': activity === 'WORKING' }"
    :data-activity="activity"
    role="img"
    :aria-label="TASK_ACTIVITY_LABEL[activity]"
    data-testid="task-activity-mark"
  >
    <span v-for="bar in 3" :key="bar" class="activity-bar" :class="BAR_COLOR[activity]"></span>
  </span>
</template>

<style scoped>
.activity-mark {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  width: 10px;
  height: 12px;
  flex-shrink: 0;
}

.activity-bar {
  width: 2px;
  height: 100%;
  border-radius: 1px;
  transform-origin: bottom;
  transform: scaleY(var(--rest));
  transition:
    transform var(--motion-move) cubic-bezier(0.2, 0.8, 0.2, 1),
    background-color var(--motion-move) ease;
}

.activity-mark[data-activity='INACTIVE'] .activity-bar {
  --rest: 0.33;
}

.activity-mark:not([data-activity='INACTIVE']) .activity-bar:nth-child(1) {
  --rest: 0.5;
}

.activity-mark:not([data-activity='INACTIVE']) .activity-bar:nth-child(2) {
  --rest: 1;
}

.activity-mark:not([data-activity='INACTIVE']) .activity-bar:nth-child(3) {
  --rest: 0.7;
}

@media (prefers-reduced-motion: no-preference) {
  .activity-mark-working .activity-bar {
    animation: activity-bar 900ms cubic-bezier(0.45, 0, 0.55, 1) infinite;
  }

  .activity-mark-working .activity-bar:nth-child(2) {
    animation-delay: -300ms;
  }

  .activity-mark-working .activity-bar:nth-child(3) {
    animation-delay: -600ms;
  }
}

@keyframes activity-bar {
  0%,
  100% {
    transform: scaleY(0.3);
  }
  50% {
    transform: scaleY(1);
  }
}
</style>
