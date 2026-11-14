<script setup lang="ts">
export type TaskRailGlyphName = 'labels' | 'context' | 'git' | 'panel'

defineProps<{ name: TaskRailGlyphName }>()
</script>

<template>
  <svg class="size-4 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <template v-if="name === 'labels'">
      <path d="M8.6 2H13a1 1 0 0 1 1 1v4.4a1 1 0 0 1-.3.7l-5.6 5.6a1 1 0 0 1-1.4 0L2.3 9.3a1 1 0 0 1 0-1.4L7.9 2.3a1 1 0 0 1 .7-.3Z" />
      <circle cx="11" cy="5" r=".8" fill="currentColor" stroke="none" />
    </template>
    <template v-else-if="name === 'context'">
      <path d="M2 11.5a6 6 0 1 1 12 0" />
      <path d="m8 11 2.4-3.4" />
    </template>
    <template v-else-if="name === 'git'">
      <circle cx="4.5" cy="3.5" r="1.6" />
      <circle cx="4.5" cy="12.5" r="1.6" />
      <circle cx="11.5" cy="5.5" r="1.6" />
      <path d="M4.5 5.1v5.8M11.5 7.1c0 2.4-2.4 2.9-7 3.4" />
    </template>
    <template v-else>
      <rect x="2" y="2.5" width="12" height="11" rx="1.5" />
      <path d="M6 2.5v11" />
    </template>
  </svg>
</template>
