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
    class="focus-ring glass absolute right-0 top-1/2 z-(--z-sticky) -translate-y-1/2 rounded-l-control border border-r-0 border-border-strong px-1.5 py-3 text-small font-medium text-text-muted shadow-lift transition-colors hover:text-anchor"
    style="writing-mode: vertical-rl"
    title="Open the chat"
    data-testid="chat-reopen-tag"
    @click="dock.open(selectedTaskId)"
  >
    Chat
  </button>
</template>
