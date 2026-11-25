<script setup lang="ts">
import { computed } from 'vue'
import { COMPOSITION_SEGMENTS } from '@/features/catalog/lib/composition-segments'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'

/** The words under a composition bar: each non-empty part with its colour and count. */
const props = defineProps<{ composition: ProjectComposition }>()

const shown = computed(() => COMPOSITION_SEGMENTS.filter((segment) => props.composition[segment.key] > 0))
</script>

<template>
  <p class="flex flex-wrap items-center gap-x-5 gap-y-1.5 text-small tabular-nums text-text-muted" data-testid="composition-legend">
    <span v-for="segment in shown" :key="segment.key" class="inline-flex items-center gap-1.5">
      <span class="size-2 rounded-key" :style="{ background: segment.tube }" aria-hidden="true" />
      <b class="font-medium" :class="segment.key === 'review' ? 'text-accent' : 'text-text'">{{ composition[segment.key] }}</b>
      {{ segment.label }}
    </span>
    <span v-if="!shown.length" class="text-text-subtle">No tasks</span>
  </p>
</template>
