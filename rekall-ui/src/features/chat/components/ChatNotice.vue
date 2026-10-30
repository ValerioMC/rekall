<script setup lang="ts">
import type { NoticeKind } from '@/features/chat/model/chat-event'

/** Something that happened to the session rather than in the conversation: a retry, a compaction. */
defineProps<{ kind: NoticeKind; text: string }>()

const TONES: Readonly<Record<NoticeKind, string>> = {
  apiRetry: 'text-warn',
  compaction: 'text-accent',
  warning: 'text-warn'
}
</script>

<template>
  <div class="flex items-center gap-3 text-caption" :class="TONES[kind]" role="status" data-testid="chat-notice" :data-kind="kind">
    <span class="h-px flex-1 bg-border" aria-hidden="true" />
    <svg class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path v-if="kind === 'compaction'" d="M2 4h8M3.5 6h5M5 8h2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <path v-else d="M6 1.8 10.6 10H1.4zM6 5v2.2M6 8.6v.1" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span class="max-w-[70%] text-center">{{ text }}</span>
    <span class="h-px flex-1 bg-border" aria-hidden="true" />
  </div>
</template>
