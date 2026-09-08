<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import ClaudeMessageBubble from '@/components/claude/ClaudeMessageBubble.vue'
import type { ClaudeMessage } from '@/model/claude'

const props = defineProps<{ messages: readonly ClaudeMessage[]; working: boolean }>()

const scroller = ref<HTMLElement | null>(null)
let stick = true

function onScroll(): void {
  const element = scroller.value
  if (!element) return
  stick = element.scrollHeight - element.scrollTop - element.clientHeight < 48
}

function toBottom(): void {
  if (!stick) return
  void nextTick(() => {
    const element = scroller.value
    if (element) element.scrollTop = element.scrollHeight
  })
}

watch(() => props.messages.length, toBottom)
watch(() => props.working, toBottom)
</script>

<template>
  <div
    ref="scroller"
    class="min-h-0 flex-1 space-y-3 overflow-y-auto px-5 py-4"
    data-testid="claude-transcript"
    @scroll="onScroll"
  >
    <ClaudeMessageBubble v-for="message in messages" :key="message.id" :message="message" />

    <div
      v-if="working"
      class="flex items-center gap-2 text-[11.5px] text-text-subtle"
      data-testid="claude-working"
    >
      <span class="relative grid size-2.5 place-items-center" aria-hidden="true">
        <span class="absolute inline-flex size-2.5 animate-ping rounded-full bg-accent/50" />
        <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
      </span>
      Claude is working
    </div>
  </div>
</template>
