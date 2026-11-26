<script setup lang="ts">
import { computed } from 'vue'
import type { ChatStatus } from '@/features/chat/lib/chat-status'

/** What a chat is doing, in words and in the colour of that state. */
const props = defineProps<{ status: ChatStatus }>()

interface StatusLook {
  readonly label: string
  readonly tone: string
  readonly pulses: boolean
}

const STATUS_LOOKS: Readonly<Record<ChatStatus, StatusLook>> = {
  connecting: { label: 'connecting', tone: 'text-accent', pulses: false },
  working: { label: 'working', tone: 'text-safe', pulses: true },
  idle: { label: 'idle', tone: 'text-safe', pulses: false },
  waiting: { label: 'waiting for approval', tone: 'text-warn', pulses: true },
  reconnecting: { label: 'reconnecting', tone: 'text-warn', pulses: false },
  ended: { label: 'ended', tone: 'text-text-subtle', pulses: false }
}

const look = computed(() => STATUS_LOOKS[props.status])
</script>

<template>
  <span
    class="inline-flex shrink-0 items-center gap-1.5 rounded-key bg-surface-raised px-2 py-0.5 text-micro font-semibold tracking-[0.02em]"
    :class="look.tone"
    data-testid="chat-status"
    :data-status="status"
  >
    <span v-if="look.pulses" class="relative grid size-2 place-items-center" aria-hidden="true">
      <span class="absolute inline-flex size-2 animate-ping rounded-full bg-current/50" />
      <span class="relative inline-flex size-1.5 rounded-full bg-current" />
    </span>
    {{ look.label }}
  </span>
</template>
