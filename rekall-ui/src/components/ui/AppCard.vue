<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{ interactive?: boolean; padded?: boolean }>(), {
  interactive: false,
  padded: true
})

const classes = computed(() => [
  'rounded-[var(--radius-card)] border border-border bg-gradient-to-b from-surface-raised to-surface shadow-[var(--shadow-raised)]',
  props.padded ? 'p-5' : '',
  props.interactive
    ? 'transition-all duration-150 hover:-translate-y-0.5 hover:border-border-strong hover:shadow-lift'
    : ''
])
</script>

<template>
  <div :class="classes"><slot /></div>
</template>
