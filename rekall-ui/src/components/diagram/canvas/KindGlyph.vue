<script setup lang="ts">
import type { DiagramKind } from '@/model/diagram'

/** One 16-unit glyph per diagram type, in the same stroke family as the element glyphs. */
withDefaults(defineProps<{ kind: DiagramKind; size?: number }>(), { size: 16 })
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <template v-if="kind === 'sequence'">
      <path d="M4 2.5v11M12 2.5v11" stroke-dasharray="1.3 1.6" />
      <path d="M4 6h7M9.6 4.6 11 6 9.6 7.4M12 10.5H5M6.4 9.1 5 10.5l1.4 1.4" />
    </template>
    <template v-else-if="kind === 'flowchart'">
      <rect x="2" y="2" width="5" height="3.6" rx="1" />
      <path d="M11.5 7.2 14 9.7l-2.5 2.5L9 9.7Z" />
      <path d="M4.5 5.6v4.1H9" />
    </template>
    <template v-else-if="kind === 'state'">
      <circle cx="3.6" cy="8" r="1.6" fill="currentColor" stroke="none" />
      <rect x="7" y="5" width="7" height="6" rx="3" />
      <path d="M5.2 8H7" />
    </template>
    <template v-else-if="kind === 'class'">
      <rect x="2.5" y="2.5" width="11" height="11" rx="1.5" />
      <path d="M2.5 6h11M2.5 9.5h11" />
    </template>
    <template v-else-if="kind === 'er'">
      <rect x="1.8" y="3" width="5" height="10" rx="1.2" />
      <rect x="9.2" y="3" width="5" height="10" rx="1.2" />
      <path d="M6.8 8h2.4M1.8 5.8h5M9.2 5.8h5" />
    </template>
    <template v-else-if="kind === 'mindmap'">
      <circle cx="8" cy="8" r="2" />
      <path d="M6.3 7 3 4.5M9.7 7 13 4.5M6.3 9 3 11.5M9.7 9 13 11.5" />
    </template>
    <template v-else-if="kind === 'timeline' || kind === 'gantt'">
      <path d="M2 13.5h12" />
      <rect x="2.5" y="3" width="6" height="2.4" rx="1" />
      <rect x="6" y="7" width="7" height="2.4" rx="1" />
    </template>
    <template v-else-if="kind === 'c4' || kind === 'architecture'">
      <rect x="2" y="2" width="12" height="12" rx="2" stroke-dasharray="2 1.6" />
      <rect x="4.6" y="4.6" width="6.8" height="6.8" rx="1.2" />
    </template>
    <template v-else>
      <rect x="2.5" y="2.5" width="11" height="11" rx="2" />
      <path d="M5 11V8M8 11V5M11 11V7" />
    </template>
  </svg>
</template>
