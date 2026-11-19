<script setup lang="ts">
import TaskRailGlyph, { type TaskRailGlyphName } from '@/features/task/components/TaskRailGlyph.vue'

defineProps<{ label: string; glyph: TaskRailGlyphName }>()
</script>

<template>
  <section class="border-b border-border px-4 pb-4 pt-3.5 last:border-b-0" :aria-label="label" :data-testid="`rail-${label.toLowerCase()}`">
    <h2 class="mb-2.5 flex items-center gap-2 text-small font-medium text-text-muted">
      <span class="text-text-subtle"><TaskRailGlyph :name="glyph" /></span>
      {{ label }}
      <span class="ml-auto font-mono text-caption font-normal text-text-subtle"><slot name="aside" /></span>
    </h2>
    <slot />
  </section>
</template>
