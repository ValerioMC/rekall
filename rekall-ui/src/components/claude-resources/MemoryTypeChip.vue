<script setup lang="ts">
import { computed } from 'vue'
import { memoryTone } from '@/model/claude-resources'

/** A memory's `type`, coloured from the diagram-kind ring so it never reads as a status or a selection. */
const props = defineProps<{ type: string | null }>()

const tint = computed(() => {
  const tone = memoryTone(props.type)
  return tone === 'none' ? 'var(--color-text-muted)' : `var(--color-kind-${tone})`
})
</script>

<template>
  <span
    v-if="type"
    class="inline-flex shrink-0 items-center rounded-key px-1.5 py-px text-micro font-medium leading-[15px]"
    :style="{ color: tint, background: `color-mix(in srgb, ${tint} 14%, transparent)`, boxShadow: `inset 0 0 0 1px color-mix(in srgb, ${tint} 30%, transparent)` }"
    data-testid="memory-type"
  >
    {{ type }}
  </span>
</template>
