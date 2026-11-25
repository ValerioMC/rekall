<script setup lang="ts">
import type { TodoStatus } from '@/features/chat/model/chat-event'

defineProps<{ status: TodoStatus }>()

const LABELS: Readonly<Record<TodoStatus, string>> = {
  pending: 'Pending',
  inProgress: 'In progress',
  completed: 'Completed'
}

const TONES: Readonly<Record<TodoStatus, string>> = {
  pending: 'text-text-subtle',
  inProgress: 'text-accent',
  completed: 'text-safe'
}
</script>

<template>
  <span class="inline-flex shrink-0" :class="TONES[status]" :data-status="status">
    <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <g stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
        <circle v-if="status === 'pending'" cx="6" cy="6" r="3.8" />
        <template v-else-if="status === 'inProgress'">
          <circle cx="6" cy="6" r="3.8" />
          <path d="M6 2.2a3.8 3.8 0 0 1 0 7.6z" fill="currentColor" />
        </template>
        <template v-else>
          <circle cx="6" cy="6" r="3.8" />
          <path d="M4.2 6.2 5.5 7.5 7.9 4.8" />
        </template>
      </g>
    </svg>
    <span class="sr-only">{{ LABELS[status] }}</span>
  </span>
</template>
