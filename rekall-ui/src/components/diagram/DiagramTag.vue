<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import type { DiagramPhase } from '@/common/diagram/task-diagrams'

/**
 * The mark a task carries for its diagrams: a comet on a ring while a session draws one (the
 * only motion, and only while that is true), a quiet badge with the count once one exists, and
 * nothing for a task that has none.
 */
defineProps<{ phase: DiagramPhase; count: number }>()
</script>

<template>
  <AppBadge v-if="phase === 'generating'" tone="accent" data-testid="diagram-tag-generating" role="status">
    <span class="generating-orbit" aria-hidden="true" />
    Generating
  </AppBadge>
  <AppBadge v-else-if="phase === 'ready'" tone="safe" data-testid="diagram-tag-ready">
    <svg viewBox="0 0 16 16" class="size-3" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <rect x="1.8" y="2.6" width="5" height="3.6" rx="1.1" />
      <rect x="9.2" y="9.8" width="5" height="3.6" rx="1.1" />
      <path d="M6.8 4.4h2a1.6 1.6 0 0 1 1.6 1.6v3.8" />
    </svg>
    {{ count > 1 ? `Diagram · ${count}` : 'Diagram' }}
  </AppBadge>
</template>

<style scoped>
.generating-orbit {
  position: relative;
  width: 9px;
  height: 9px;
  border-radius: 999px;
  border: 1.4px solid color-mix(in srgb, var(--color-accent) 28%, transparent);
}

.generating-orbit::after {
  content: '';
  position: absolute;
  inset: -1.4px;
  border-radius: 999px;
  border: 1.4px solid transparent;
  border-top-color: var(--color-accent);
  animation: orbit 1.1s linear infinite;
  filter: drop-shadow(0 0 3px var(--color-accent));
}

@keyframes orbit {
  to {
    transform: rotate(360deg);
  }
}
</style>
