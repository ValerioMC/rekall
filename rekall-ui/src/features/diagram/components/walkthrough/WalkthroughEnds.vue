<script setup lang="ts">
import { computed } from 'vue'
import ElementGlyph from '@/features/diagram/components/canvas/ElementGlyph.vue'
import { EXTERNAL_STYLE } from '@/features/diagram/lib/element-style'
import type { SequenceMessage } from '@/features/diagram/lib/scene/sequence-scene'

/** The two ends of a call: who makes it and who receives it, an external system marked as one. */
const props = defineProps<{ message: SequenceMessage; labels: ReadonlyMap<string, string>; external: ReadonlySet<string> }>()

const ends = computed(() =>
  [props.message.from, props.message.to].map((id) => ({ id, label: props.labels.get(id) ?? id, external: props.external.has(id) }))
)
</script>

<template>
  <span class="flex min-w-0 flex-wrap items-center gap-1.5 text-small">
    <template v-for="(end, index) in ends" :key="`${end.id}-${index}`">
      <svg v-if="index === 1" viewBox="0 0 16 16" class="size-3.5 shrink-0 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-label="to"><path d="M3 8h9.5M9 4.5 12.5 8 9 11.5" /></svg>
      <span class="inline-flex min-w-0 items-center gap-1 rounded-control border border-border bg-canvas px-1.5 py-0.5">
        <ElementGlyph v-if="end.external" :glyph="EXTERNAL_STYLE.glyph" :size="13" :style="{ color: EXTERNAL_STYLE.tint }" />
        <span class="truncate text-text">{{ end.label }}</span>
        <span v-if="end.external" class="text-micro text-text-subtle">external</span>
      </span>
    </template>
  </span>
</template>
