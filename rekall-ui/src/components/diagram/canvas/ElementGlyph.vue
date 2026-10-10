<script setup lang="ts">
import type { ElementGlyph } from '@/common/diagram/element-style'

/**
 * One 16-unit glyph per element, stroked in `currentColor` at one weight, so a row of them reads
 * as one family. A nested `<svg>`: it sits inside a canvas card or inline in HTML alike.
 */
withDefaults(defineProps<{ glyph: ElementGlyph; size?: number; x?: number; y?: number }>(), { size: 16, x: 0, y: 0 })
</script>

<template>
  <svg :x="x" :y="y" :width="size" :height="size" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <template v-if="glyph === 'participant'">
      <rect x="2.6" y="2.2" width="10.8" height="5.2" rx="1.4" />
      <path d="M8 7.4v6.4" stroke-dasharray="1.4 1.6" />
    </template>
    <template v-else-if="glyph === 'actor'">
      <circle cx="8" cy="4.2" r="2.2" />
      <path d="M8 6.4v4.4M4.4 8.4h7.2M8 10.8l-2.8 3.4M8 10.8l2.8 3.4" />
    </template>
    <path v-else-if="glyph === 'step'" d="M2.8 8h8.4M8.2 4.6 11.6 8l-3.4 3.4" />
    <template v-else-if="glyph === 'decision'">
      <path d="M8 2.4 13.6 8 8 13.6 2.4 8Z" />
      <path d="M8 6v2.2" />
      <circle cx="8" cy="10.2" r="0.4" fill="currentColor" />
    </template>
    <rect v-else-if="glyph === 'terminal'" x="2" y="5" width="12" height="6" rx="3" />
    <template v-else-if="glyph === 'data'">
      <ellipse cx="8" cy="4" rx="5" ry="1.9" />
      <path d="M3 4v8c0 1 2.2 1.9 5 1.9s5-.9 5-1.9V4M3 8c0 1 2.2 1.9 5 1.9S13 9 13 8" />
    </template>
    <template v-else-if="glyph === 'state'">
      <circle cx="8" cy="8" r="5.4" />
      <circle cx="8" cy="8" r="2" fill="currentColor" stroke="none" />
    </template>
    <circle v-else-if="glyph === 'pseudo'" cx="8" cy="8" r="3.4" fill="currentColor" stroke="none" />
    <template v-else-if="glyph === 'class'">
      <rect x="2.4" y="2.4" width="11.2" height="11.2" rx="1.6" />
      <path d="M2.4 6.2h11.2M2.4 9.8h11.2" />
    </template>
    <template v-else-if="glyph === 'entity'">
      <rect x="2.4" y="2.4" width="11.2" height="11.2" rx="1.6" />
      <path d="M2.4 6h11.2M6.4 6v7.6" />
    </template>
    <template v-else-if="glyph === 'note'">
      <path d="M3 2.4h7l3 3v8.2H3Z" />
      <path d="M10 2.4v3h3M5.4 8.4h5.2M5.4 11h3.6" />
    </template>
    <path v-else d="M2.4 8h10M9.4 5 12.4 8l-3 3" />
  </svg>
</template>
