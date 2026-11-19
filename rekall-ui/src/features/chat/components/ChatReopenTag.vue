<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'

/** The edge tag that brings the chat window back while it is hidden and the selected task has a session. */
const chat = useChatStore()
const dock = useChatDockStore()
const { selectedTaskId } = storeToRefs(useConsoleStore())

const shown = computed(() => dock.hidden && chat.sessionFor(selectedTaskId.value) !== null)
</script>

<template>
  <button
    v-if="shown"
    type="button"
    class="focus-ring group absolute right-0 top-1/2 z-(--z-sticky) flex -translate-y-1/2 cursor-pointer flex-col items-center gap-2.5 rounded-l-card border border-r-0 border-border-strong bg-surface-raised py-3 pl-3 pr-2 text-text-muted shadow-raised transition-colors duration-(--motion-move) hover:bg-surface-hover hover:text-text"
    title="Open the chat"
    data-testid="chat-reopen-tag"
    @click="dock.open(selectedTaskId)"
  >
    <span
      class="absolute inset-y-2.5 left-0 w-[3px] rounded-r-full bg-accent opacity-70 transition-opacity duration-(--motion-move) group-hover:opacity-100"
      aria-hidden="true"
    />
    <svg class="size-3.5 shrink-0 text-accent" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path
        d="M2.5 2h9A1.5 1.5 0 0 1 13 3.5v5A1.5 1.5 0 0 1 11.5 10H6.2L3.6 12.2a.4.4 0 0 1-.65-.3V10h-.45A1.5 1.5 0 0 1 1 8.5v-5A1.5 1.5 0 0 1 2.5 2Z"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linejoin="round"
      />
    </svg>
    <span class="text-small font-semibold tracking-[0.04em]" style="writing-mode: vertical-rl">Chat</span>
  </button>
</template>
