<script setup lang="ts">
import { SOURCE_LABEL } from '@/features/decisions/lib/decision-labels'
import type { DecisionSource } from '@/features/decisions/model/decision'

/** Where a decision was taken, as a small mark and its name. */
defineProps<{ source: DecisionSource }>()
</script>

<template>
  <span class="inline-flex items-center gap-1.5" :class="source === 'PERSON' ? 'text-session-strong' : 'text-text-muted'" data-testid="decision-source">
    <svg viewBox="0 0 16 16" class="size-3 shrink-0" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <template v-if="source === 'TERMINAL'">
        <rect x="2" y="3" width="12" height="10" rx="1.5" />
        <path d="m5 6.5 2 1.5-2 1.5M8.5 10h2.5" />
      </template>
      <path v-else-if="source === 'CHAT'" d="M2.5 3.5h11v7h-6l-3 2.5v-2.5h-2z" />
      <path v-else d="M10.5 2.8 13.2 5.5 5.5 13.2 2.5 13.5 2.8 10.5Z" />
    </svg>
    {{ SOURCE_LABEL[source] }}
  </span>
</template>
