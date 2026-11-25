<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import TaskActivityMark from '@/features/workingset/components/TaskActivityMark.vue'
import { chatStatus, type ChatStatus } from '@/features/chat/lib/chat-status'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The handle that brings the chat window back while it is hidden and the selected task has a
 * session. It carries the session's state, from the events the hidden window keeps following:
 * moving bars while Claude works, an amber pill while it waits for an answer, a still glyph otherwise.
 */
interface HandleLook {
  readonly name: string
  readonly tone: string
}

const HANDLE_LOOKS: Readonly<Record<ChatStatus, HandleLook>> = {
  working: {
    name: 'Open the chat, Claude is working',
    tone: 'border-session bg-surface-raised text-session-strong hover:bg-surface-hover'
  },
  waiting: {
    name: 'Open the chat, waiting for your answer',
    tone: 'border-accent-deep bg-accent text-accent-ink hover:bg-accent-strong'
  },
  idle: {
    name: 'Open the chat, Claude is idle',
    tone: 'border-border-strong bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text'
  },
  ended: {
    name: 'Open the chat, the session has ended',
    tone: 'border-dashed border-border-strong bg-surface text-text-subtle hover:bg-surface-hover hover:text-text'
  },
  connecting: {
    name: 'Open the chat',
    tone: 'border-border-strong bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text'
  },
  reconnecting: {
    name: 'Open the chat',
    tone: 'border-border-strong bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text'
  }
}

const chat = useChatStore()
const dock = useChatDockStore()
const { selectedTaskId } = storeToRefs(useConsoleStore())

const session = computed(() => chat.sessionFor(selectedTaskId.value))
const shown = computed(() => dock.hidden && session.value !== null)
const status = computed<ChatStatus>(() => {
  const current = session.value
  if (current === null) return 'idle'
  return chatStatus({ timeline: chat.timelineOf(current.id), session: current, connected: true, reconnecting: false })
})
const look = computed(() => HANDLE_LOOKS[status.value])
</script>

<template>
  <Transition name="chat-handle">
    <button
      v-if="shown"
      type="button"
      class="chat-handle focus-ring absolute bottom-4 right-4 z-(--z-sticky) flex h-10 cursor-pointer items-center rounded-full border text-text shadow-raised"
      :class="look.tone"
      :aria-label="look.name"
      :title="look.name"
      :data-status="status"
      data-testid="chat-reopen-tag"
      @click="dock.open(selectedTaskId)"
    >
      <span
        v-if="status === 'waiting'"
        class="absolute inset-0 -z-10 rounded-full bg-accent/40 motion-safe:animate-ping"
        aria-hidden="true"
        data-testid="chat-reopen-ping"
      />
      <span class="relative grid size-10 shrink-0 place-items-center" aria-hidden="true">
        <span v-if="status === 'working'" class="grid scale-150 place-items-center">
          <TaskActivityMark activity="WORKING" />
        </span>
        <svg v-else class="size-4" viewBox="0 0 14 14" fill="none">
          <path
            d="M2.5 2h9A1.5 1.5 0 0 1 13 3.5v5A1.5 1.5 0 0 1 11.5 10H6.2L3.6 12.2a.4.4 0 0 1-.65-.3V10h-.45A1.5 1.5 0 0 1 1 8.5v-5A1.5 1.5 0 0 1 2.5 2Z"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linejoin="round"
          />
        </svg>
        <span
          v-if="status === 'idle'"
          class="absolute right-2 top-2 size-1.5 rounded-full bg-safe"
          data-testid="chat-reopen-idle-dot"
        />
      </span>
      <span class="chat-handle-label" aria-hidden="true">
        <span class="min-w-0 overflow-hidden whitespace-nowrap">
          <span class="block pr-4 text-small font-semibold" data-testid="chat-reopen-label">Needs your answer</span>
        </span>
      </span>
    </button>
  </Transition>
</template>

<style scoped>
.chat-handle {
  transition:
    background-color var(--motion-quick) ease,
    color var(--motion-quick) ease;
}

.chat-handle-enter-active,
.chat-handle-leave-active {
  transition:
    transform var(--motion-move) cubic-bezier(0.25, 1, 0.5, 1),
    opacity var(--motion-move) cubic-bezier(0.25, 1, 0.5, 1);
}

.chat-handle-enter-from,
.chat-handle-leave-to {
  opacity: 0;
  transform: translateX(calc(100% + 1rem));
}

.chat-handle-label {
  display: grid;
  grid-template-columns: 0fr;
  opacity: 0;
  transition:
    grid-template-columns var(--motion-move) cubic-bezier(0.25, 1, 0.5, 1),
    opacity var(--motion-move) ease;
}

.chat-handle[data-status='waiting'] .chat-handle-label {
  grid-template-columns: 1fr;
  opacity: 1;
}
</style>
