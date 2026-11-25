<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * Owns the chat window's slide in and out. Once shown, or once the selected task has a session, the
 * window stays mounted, hidden and inert: its draft and scroll survive a hide and its events keep
 * the reopen handle's state live. Loaded only once a chat session exists, like the terminal pane.
 */
const props = defineProps<{ shown: boolean }>()

const ChatWindow = defineAsyncComponent(() => import('@/features/chat/components/ChatWindow.vue'))
const dock = useChatDockStore()
const chat = useChatStore()
const { selectedTaskId } = storeToRefs(useConsoleStore())
const shownOnce = ref(props.shown)
const mounted = computed(() => shownOnce.value || chat.sessionFor(selectedTaskId.value) !== null)

function returnFocusToConsole(): void {
  const focused = document.activeElement
  if (focused instanceof HTMLElement && focused.closest('.chat-dock')) document.getElementById('note')?.focus()
}

watch(
  () => props.shown,
  (shown) => {
    if (shown) shownOnce.value = true
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
