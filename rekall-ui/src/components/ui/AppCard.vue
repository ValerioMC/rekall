<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{ interactive?: boolean; padded?: boolean }>(), {
  interactive: false,
  padded: true
})

const classes = computed(() => [
  'rounded-[var(--radius-card)] border border-border bg-surface',
  props.padded ? 'p-5' : '',
  // A one pixel lift and a brighter edge: enough to read as clickable without animating the layout.
  props.interactive
    ? 'transition-all duration-150 hover:-translate-y-0.5 hover:border-border-strong hover:bg-surface-raised hover:shadow-lift'
    : ''
])
</script>

<template>
  <div :class="classes"><slot /></div>
</template>
