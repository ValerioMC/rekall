<script setup lang="ts">
import { defineAsyncComponent, ref, watch } from 'vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'

/**
 * Owns the chat window's slide in and out. Once shown the window stays mounted, hidden and inert,
 * so its draft and scroll survive a hide. Loaded only once a chat session exists, like the terminal pane.
 */
const props = defineProps<{ shown: boolean }>()

const ChatWindow = defineAsyncComponent(() => import('@/features/chat/components/ChatWindow.vue'))
const dock = useChatDockStore()
const mounted = ref(props.shown)

function returnFocusToConsole(): void {
  const focused = document.activeElement
  if (focused instanceof HTMLElement && focused.closest('.chat-dock')) document.getElementById('note')?.focus()
}

watch(
  () => props.shown,
  (shown) => {
    if (shown) mounted.value = true
    else returnFocusToConsole()
  },
  { flush: 'sync' }
)
</script>

<template>
  <ChatWindow
    v-if="mounted"
    class="chat-dock"
    :style="{ '--chat-dock-width': `${dock.shownWidth}px` }"
    :data-state="shown ? 'shown' : 'hidden'"
    :inert="shown ? undefined : true"
  />
</template>
