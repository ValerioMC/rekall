<script setup lang="ts">
import { computed } from 'vue'
import KindGlyph from '@/features/diagram/components/canvas/KindGlyph.vue'
import KindSketch from './KindSketch.vue'
import { KIND_INFO } from '@/features/diagram/lib/kinds'
import { relativeTime } from '@/common/format/relative-time'
import type { DiagramSummary } from '@/features/diagram/model/diagram'

/** One diagram in the gallery: a sketch of its first sheet, what it answers, its sheets' types, and whether it plays as a walkthrough. */
const props = defineProps<{ summary: DiagramSummary; taskAnchor: string | null; arrived: boolean }>()

const emit = defineEmits<{ open: [] }>()

const firstKind = computed(() => props.summary.kinds[0] ?? 'flowchart')
</script>

<template>
  <button
    class="diagram-card focus-ring group flex min-w-0 flex-col overflow-hidden rounded-card border border-border bg-surface text-left shadow-(--shadow-raised) transition-[border-color,transform] duration-(--motion-quick) hover:-translate-y-px hover:border-border-strong"
    :class="{ 'is-arrived': arrived }"
    data-testid="diagram-card"
    @click="emit('open')"
  >
    <span class="diagram-card-sketch relative block h-[92px] border-b border-border bg-canvas px-2 pt-1.5">
      <KindSketch :kind="firstKind" />
      <span v-if="summary.walkthrough" class="absolute left-2 top-2 inline-flex items-center gap-1 rounded-full border border-accent/40 bg-canvas px-2 py-0.5 text-micro font-semibold text-accent" data-testid="diagram-card-walkthrough">
        <svg viewBox="0 0 16 16" class="size-2.5" fill="currentColor" aria-hidden="true"><path d="M5 3.3v9.4c0 .5.5.8.9.5l7-4.7c.4-.3.4-.8 0-1.1l-7-4.6c-.4-.3-.9 0-.9.5Z" /></svg>
        Walkthrough
      </span>
      <span v-if="arrived" class="absolute right-2 top-2 rounded-full bg-accent-soft px-2 py-0.5 text-micro font-semibold text-accent" role="status">New</span>
    </span>
    <span class="flex flex-1 flex-col gap-1.5 px-3.5 pb-3 pt-3">
      <span class="line-clamp-2 text-body font-semibold leading-snug tracking-[-0.005em] text-text">{{ summary.title }}</span>
      <span v-if="summary.question" class="line-clamp-2 text-small leading-snug text-text-muted">{{ summary.question }}</span>
      <span class="mt-auto flex flex-wrap items-center gap-1.5 pt-1.5">
        <span
          v-for="(kind, index) in summary.kinds"
          :key="`${kind}-${index}`"
          class="inline-flex items-center gap-1 rounded-full border border-border bg-canvas px-1.5 py-0.5 text-micro text-text-muted"
        >
          <KindGlyph :kind="kind" :size="12" />
          {{ KIND_INFO[kind].label }}
        </span>
        <span v-if="taskAnchor" class="anchor-chip truncate px-1.5 py-0.5 text-micro">{{ taskAnchor }}</span>
        <span class="ml-auto shrink-0 text-micro tabular-nums text-text-subtle" :title="summary.updatedAt">{{ relativeTime(summary.updatedAt) }}</span>
      </span>
    </span>
  </button>
</template>

<style scoped>
/* A diagram a session just delivered wears the accent once, then settles like the rest. */
.is-arrived {
  border-color: color-mix(in srgb, var(--color-accent) 60%, transparent);
  animation: card-arrived 1.6s ease-out 1;
}

@keyframes card-arrived {
  from {
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--color-accent) 28%, transparent);
  }
}

@media (prefers-reduced-motion: reduce) {
  .is-arrived {
    animation: none;
  }
}
</style>
