<script setup lang="ts">
import type { KindStyle } from '@/common/diagram/kind-style'

/**
 * One 16-unit glyph per kind, stroked in `currentColor` at the same weight, so a row of them
 * reads as one family. A nested `<svg>`: it sits inside a canvas node or inline in HTML alike.
 */
withDefaults(defineProps<{ glyph: KindStyle['glyph']; size?: number; x?: number; y?: number }>(), {
  size: 16,
  x: 0,
  y: 0
})
</script>

<template>
  <svg
    :x="x"
    :y="y"
    :width="size"
    :height="size"
    viewBox="0 0 16 16"
    fill="none"
    stroke="currentColor"
    stroke-width="1.4"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path v-if="glyph === 'concept'" d="M8 2.2 9.5 6.5 13.8 8 9.5 9.5 8 13.8 6.5 9.5 2.2 8 6.5 6.5Z" />
    <path v-else-if="glyph === 'action'" d="M2.8 8h8.4M8.2 4.6 11.6 8l-3.4 3.4" />
    <template v-else-if="glyph === 'decision'">
      <path d="M8 2.4 13.6 8 8 13.6 2.4 8Z" />
      <path d="M8 6v2.2" />
      <circle cx="8" cy="10.2" r="0.4" fill="currentColor" />
    </template>
    <template v-else-if="glyph === 'state'">
      <circle cx="8" cy="8" r="5.4" />
      <circle cx="8" cy="8" r="2" fill="currentColor" stroke="none" />
    </template>
    <path v-else-if="glyph === 'event'" d="M8.9 1.9 3.9 9h4l-.8 5.1L12.1 7h-4Z" />
    <template v-else-if="glyph === 'data'">
      <ellipse cx="8" cy="4" rx="5" ry="1.9" />
      <path d="M3 4v8c0 1 2.2 1.9 5 1.9s5-.9 5-1.9V4M3 8c0 1 2.2 1.9 5 1.9S13 9 13 8" />
    </template>
    <template v-else-if="glyph === 'external_system'">
      <path d="M9.5 2.8H13.2V6.5M13.2 2.8 7.6 8.4" />
      <path d="M11.8 9.4v2.9a1 1 0 0 1-1 1H3.7a1 1 0 0 1-1-1V5.2a1 1 0 0 1 1-1h2.9" />
    </template>
    <path v-else-if="glyph === 'code'" d="M5.6 4.4 2 8l3.6 3.6M10.4 4.4 14 8l-3.6 3.6M9.1 3 6.9 13" />
    <rect v-else x="3.2" y="3.2" width="9.6" height="9.6" rx="2.4" />
  </svg>
</template>
