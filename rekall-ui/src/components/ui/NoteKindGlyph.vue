<script setup lang="ts">
import { computed } from 'vue'

/** A note's kind as a 12px glyph, in the slot a task row gives its status mark. */
const props = defineProps<{ kind: string }>()

const PATHS: Readonly<Record<string, string>> = {
  context: 'M6 1.5v2M6 8.5v2M1.5 6h2M8.5 6h2M6 3.5a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5Z',
  notes: 'M2.5 1.5h7v9h-7zM4.3 4h3.4M4.3 6h3.4M4.3 8h2',
  architecture: 'M1.5 1.5h4v4h-4zM6.5 6.5h4v4h-4zM3.5 5.5v3h3M8.5 6.5v-3h-3',
  report: 'M1.5 10.5h9M3 10.5V6.5M6 10.5V2.5M9 10.5V5'
}
const FALLBACK = 'M6 2.2a3.8 3.8 0 1 1 0 7.6 3.8 3.8 0 0 1 0-7.6Z'

const path = computed(() => PATHS[props.kind] ?? FALLBACK)
</script>

<template>
  <svg class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" role="img">
    <title>{{ kind }}</title>
    <path :d="path" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
  </svg>
</template>
