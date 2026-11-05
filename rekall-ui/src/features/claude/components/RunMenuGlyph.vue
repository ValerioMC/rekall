<script setup lang="ts">
import type { RunMenuAction } from '@/features/claude/model/runMenu'

defineProps<{ action: RunMenuAction }>()
</script>

<template>
  <svg class="size-4 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <g stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
      <template v-if="action === 'RUN_IN_QUEUE'">
        <path d="M2.5 4h7M2.5 8h5M2.5 12h5" />
        <path d="m10.5 8.2 3 1.8-3 1.8Z" />
      </template>
      <template v-else-if="action === 'PLAN'">
        <path d="m2.5 4 1.2 1.2L5.7 3M2.5 9l1.2 1.2L5.7 8" />
        <path d="M8 4.2h5.5M8 9.2h5.5M8 13h5.5" />
      </template>
      <template v-else-if="action === 'PLAN_FLOW'">
        <circle cx="4" cy="3.5" r="1.5" />
        <circle cx="4" cy="12.5" r="1.5" />
        <circle cx="12" cy="8" r="1.5" />
        <path d="M4 5v6M5.4 4.2 10.6 7M5.4 11.8 10.6 9" />
      </template>
      <template v-else>
        <path d="M13 8a5 5 0 1 1-1.6-3.7" />
        <path d="M13 2.8v2.7h-2.7" />
      </template>
    </g>
  </svg>
</template>
