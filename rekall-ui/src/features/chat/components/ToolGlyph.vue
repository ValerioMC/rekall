<script setup lang="ts">
import type { ToolIcon } from '@/features/chat/lib/describe-tool'

defineProps<{ icon: ToolIcon }>()
</script>

<template>
  <svg class="size-3.5 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
    <g stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
      <template v-if="icon === 'terminal'">
        <path d="M2 3.5 5 6 2 8.5" />
        <path d="M6.5 9H10" />
      </template>
      <path v-else-if="icon === 'pencil'" d="M8.3 1.9 10.1 3.7 4.3 9.5 2 10 2.5 7.7z" />
      <template v-else-if="icon === 'file-plus'">
        <path d="M7 1.5H3.5a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1V4z" />
        <path d="M6 5.5v3M4.5 7h3" />
      </template>
      <template v-else-if="icon === 'file'">
        <path d="M7 1.5H3.5a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1V4z" />
        <path d="M7 1.5V4h2.5" />
      </template>
      <template v-else-if="icon === 'search'">
        <circle cx="5.2" cy="5.2" r="3" />
        <path d="m7.5 7.5 2.8 2.8" />
      </template>
      <template v-else>
        <circle cx="6" cy="6" r="1.6" />
        <path d="M6 1.5v1.4M6 9.1v1.4M1.5 6h1.4M9.1 6h1.4" />
      </template>
    </g>
  </svg>
</template>
