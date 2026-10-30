<script setup lang="ts">
import { computed } from 'vue'
import type { ToolState } from '@/features/chat/model/timeline'

const props = defineProps<{ state: ToolState }>()

const LABELS: Readonly<Record<ToolState, string>> = {
  running: 'Running',
  done: 'Done',
  failed: 'Failed',
  denied: 'Denied'
}

const TONES: Readonly<Record<ToolState, string>> = {
  running: 'text-text-muted',
  done: 'text-safe',
  failed: 'text-danger',
  denied: 'text-warn'
}

const label = computed(() => LABELS[props.state])
</script>

<template>
  <span class="inline-flex shrink-0 items-center" :class="TONES[state]" :data-state="state">
    <svg class="size-3.5" :class="state === 'running' && 'animate-spin'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <g stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <path v-if="state === 'running'" d="M10 6a4 4 0 1 1-4-4" />
        <path v-else-if="state === 'done'" d="M2.3 6.3 4.8 8.8 9.8 3.4" />
        <path v-else-if="state === 'failed'" d="m3 3 6 6M9 3 3 9" />
        <template v-else>
          <circle cx="6" cy="6" r="4" />
          <path d="m3.2 8.8 5.6-5.6" />
        </template>
      </g>
    </svg>
    <span class="sr-only">{{ label }}</span>
  </span>
</template>
